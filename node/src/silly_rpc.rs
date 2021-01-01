use jsonrpc_derive::rpc;
use jsonrpc_core::Error;

#[rpc]
pub trait SillyRpc {
    #[rpc(name = "silly_seven")]
    fn silly_7(&self) -> Result<u64, Error>;

    #[rpc(name = "silly_double")]
    fn silly_double(&self, val: u64) -> Result<u64, Error>;
}

pub struct Silly;

impl SillyRpc for Silly {
    fn silly_7(&self) -> Result<u64, Error> {
        Ok((7))
    }

    fn silly_double(&self, val: u64) -> Result<u64, Error> {
        Ok(2 * val)
    }
}