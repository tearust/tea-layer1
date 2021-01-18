//! RPC interface for the transaction payment module.

use tea_runtime_api::TeaApi as TeaRuntimeApi;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait TeaApi<BlockHash> {
    #[rpc(name = "tea_getDelegates")]
    fn get_delegates(&self, start: u32, count: u32, at: Option<BlockHash>)
        -> Result<Vec<([u8; 32], [u8; 32])>>;
}

/// A struct that implements the `TeaApi`.
pub struct Tea<C, M> {
    // If you have more generics, no need to SumStorage<C, M, N, P, ...>
    // just use a tuple like SumStorage<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Tea<C, M> {
    /// Create new `tea` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> TeaApi<<Block as BlockT>::Hash> for Tea<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: TeaRuntimeApi<Block>,
{
    fn get_delegates(
        &self,
        start: u32,
        count: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<([u8; 32], [u8; 32])>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.get_delegates(&at, start, count);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
