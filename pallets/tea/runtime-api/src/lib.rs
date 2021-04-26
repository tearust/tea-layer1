#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
#[allow(clippy::unnecessary_mut_passed)]
sp_api::decl_runtime_apis! {
    pub trait TeaApi {
        fn get_delegates(start: u32, count: u32) -> Vec<(Vec<u8>, [u8; 32], Vec<u8>)>;
    }
}
