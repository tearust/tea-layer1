use jsonrpc_derive::rpc;
use jsonrpc_core::{IoHandler, Error as RpcError, Result, ErrorCode};
use jsonrpc_core::futures::future::{self, FutureResult};
use std::sync::Arc;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, HashFor, NumberFor, Hash}
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use tea_runtime::tea::TeaApi;

// #[rpc]
// pub trait SillyRpc {
//     #[rpc(name = "silly_seven")]
//     fn silly_7(&self) -> Result<u64>;
//
//     #[rpc(name = "silly_double")]
//     fn silly_double(&self, val: u64) -> Result<u64>;
// }
//
// pub struct Silly;
//
// impl SillyRpc for Silly {
//     fn silly_7(&self) -> Result<u64> {
//         Ok(7)
//     }
//
//     fn silly_double(&self, val: u64) -> Result<u64> {
//         Ok(2 * val)
//     }
// }

#[rpc]
pub trait TeaNodeApi<BlockHash> {
    #[rpc(name = "tea_getSum")]
    fn get_sum(
        &self,
        at: Option<BlockHash>,
    ) -> Result<u32>;
}

/// A struct that implements the `TeaApi`.
pub struct TeaNode<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> TeaNode<C, M> {
    /// Create new `Tea` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block> TeaNodeApi<<Block as BlockT>::Hash> for TeaNode<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: TeaApi<Block>,
{
    fn get_sum(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<u32> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        ));

        let runtime_api_result = api.get_sum(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}