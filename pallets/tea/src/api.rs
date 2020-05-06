use sp_std::vec::Vec;
use frame_support::sp_runtime::RuntimeString;
use crate::Node;

sp_api::decl_runtime_apis! {
    pub trait TeaApi {
        fn get_sum() -> u32;
        fn get_node(key: Vec<u8>) -> Node;
        fn get_bootstrap() -> Vec<RuntimeString>;
    }
}
