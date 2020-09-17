use jsonrpc_derive::rpc;
use jsonrpc_core::{Error as RpcError, Result, ErrorCode};
use std::sync::Arc;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, Hash}
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use tea_runtime::tea::api::TeaApi;
use tea_runtime::tea::TeaPubKey;
use std::vec::Vec;
use hex::FromHex;

// #[rpc]
// pub trait TeaNodeApi<BlockHash> {
//     #[rpc(name = "tea_getSum")]
//     fn get_sum(
//         &self,
//         at: Option<BlockHash>,
//     ) -> Result<u32>;
//
//     #[rpc(name = "tea_getNodeByEphemeralId")]
//     fn get_node_by_ephemeral_id(
//         &self,
//         id_hex: String,
//         at: Option<BlockHash>,
//     ) -> Result<Option<tea_runtime::tea::Node>>;
// }
//
// /// A struct that implements the `TeaApi`.
// pub struct TeaNode<C, M> {
//     client: Arc<C>,
//     _marker: std::marker::PhantomData<M>,
// }
//
// impl<C, M> TeaNode<C, M> {
//     /// Create new `Tea` instance with the given reference to the client.
//     pub fn new(client: Arc<C>) -> Self {
//         Self { client, _marker: Default::default() }
//     }
// }
//
// impl<C, Block> TeaNodeApi<<Block as BlockT>::Hash> for TeaNode<C, Block>
//     where
//         Block: BlockT,
//         C: Send + Sync + 'static,
//         C: ProvideRuntimeApi<Block>,
//         C: HeaderBackend<Block>,
//         C::Api: TeaApi<Block>,
// {
//     fn get_sum(
//         &self,
//         at: Option<<Block as BlockT>::Hash>,
//     ) -> Result<u32> {
//         let api = self.client.runtime_api();
//         let at = BlockId::hash(at.unwrap_or_else(||
//             // If the block hash is not supplied assume the best block.
//             self.client.info().best_hash
//         ));
//
//         let runtime_api_result = api.get_sum(&at);
//         runtime_api_result.map_err(|e| RpcError {
//             code: ErrorCode::ServerError(9876), // No real reason for this value
//             message: "Something wrong".into(),
//             data: Some(format!("{:?}", e).into()),
//         })
//     }
//
//     fn get_node_by_ephemeral_id(
//         &self,
//         id_hex: String,
//         at: Option<<Block as BlockT>::Hash>,
//     ) -> Result<Option<tea_runtime::tea::Node>> {
//         let api = self.client.runtime_api();
//         let at = BlockId::hash(at.unwrap_or_else(||
//             // If the block hash is not supplied assume the best block.
//             self.client.info().best_hash
//         ));
//
//         let key = Vec::from_hex(id_hex).map_err(|e| RpcError{
//             code: ErrorCode::ServerError(9875), // No real reason for this value
//             message: "Invalid key hex string.".into(),
//             data: Some(format!("{:?}", e).into()),
//         })?;
//
//         let mut k = [0u8; 32];
//         k.copy_from_slice(key.as_slice());
//
//         let runtime_api_result = api.get_node_by_ephemeral_id(&at, k);
//         runtime_api_result.map_err(|e| RpcError {
//             code: ErrorCode::ServerError(9876), // No real reason for this value
//             message: "Get node info failed.".into(),
//             data: Some(format!("{:?}", e).into()),
//         })
//     }
// }