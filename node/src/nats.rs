use nats;
use std::sync::Arc;
use sp_api::ProvideRuntimeApi;
use tea_runtime::tea::api::TeaApi;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, Hash},
};
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use serde_json;
use std::vec::Vec;
use hex::FromHex;

pub struct NatsServer<C, M> {
    _client: std::marker::PhantomData<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, Block> NatsServer<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: TeaApi<Block>,
{
    pub fn start_nats_service(client: Arc<C>) -> std::io::Result<()> {
        let nc = nats::connect("localhost")?;

        Self::sub_node_info(client.clone(), &nc)?;
        Self::layer1_async(client, nc)?;

        Ok(())
    }

    fn sub_node_info(client: Arc<C>, nc: &nats::Connection) -> std::io::Result<()> {
        let subject = String::from("layer1.node_info");
        let sub = nc.subscribe(&subject)?;
        std::thread::spawn(move || {
            for msg in sub.messages() {
                println!("Received a request {}", msg);
                let api = client.runtime_api();
                let at = BlockId::hash(client.info().best_hash);
                let key_vec = hex_to_vec(msg.data.clone());
                match key_vec {
                    Ok(key) => {
                        let node_info = api.get_node(&at, key).unwrap();
                        msg.respond(serde_json::to_vec(&node_info).unwrap());
                    }
                    Err(e) => {
                        msg.respond(e);
                    }
                }
            }
        });
        println!("Listening for requests on '{}'", subject);

        Ok(())
    }

    fn layer1_async(client: Arc<C>, nc: nats::Connection) -> std::io::Result<()> {
        let subject = String::from("layer1.async.*.>");
        let sub = nc.subscribe(&subject)?;
        std::thread::spawn(move || ->std::io::Result<()> {
            for msg in sub.messages() {
                println!("Received a request {}", msg);
                // Subject should follow the format 'layer1.async.{reply_to}.{action}'.
                let sub_sections: Vec<_> = msg.subject.split('.').collect();
                if sub_sections.len() < 4 {
                    println!("Invalid subject format");
                    continue;
                }
                let reply_to = sub_sections[2];
                let action = sub_sections[3];
                let api = client.runtime_api();
                let at = BlockId::hash(client.info().best_hash);
                match action {
                    "bootstrap" => {
                        let bootstrap = api.get_bootstrap(&at).unwrap();
                        let reply_to_data = format!("{}.action.{}", reply_to, action);
                        nc.publish(reply_to_data.as_str(), serde_json::to_vec(&bootstrap).unwrap())?;
                    }
                    "node_info" => {
                        let key_vec = hex_to_vec(msg.data.clone());
                        match key_vec {
                            Ok(key) => {
                                let node_info = api.get_node(&at, key).unwrap();
                                let reply_to_data = format!("{}.action.{}", reply_to, action);
                                nc.publish(reply_to_data.as_str(), serde_json::to_vec(&node_info).unwrap())?;
                            }
                            Err(e) => {
                                let reply_to_error = format!("{}.error.invalid_node_key", reply_to);
                                nc.publish(reply_to_error.as_str(), "")?;
                            }
                        }
                    }
                    _ => {
                        let reply_to_error = format!("{}.error.action_{}_dose_not_support", reply_to, action);
                        nc.publish(reply_to_error.as_str(), "")?;
                    }
                }
            }
            Ok(())
        });

        println!("Listening for requests on '{}'", subject);
        Ok(())
    }
}

fn hex_to_vec(hex: Vec<u8>) -> Result<Vec<u8>, String> {
    let key_hex = String::from_utf8(hex);
    return match key_hex {
        Ok(hex) => {
            match Vec::from_hex(hex) {
                Ok(vec) => Ok(vec),
                Err(e) => {
                    Err(format!("{:?}", e))
                }
            }
        }
        Err(e) => {
            Err(format!("{:?}", e))
        }
    };
}