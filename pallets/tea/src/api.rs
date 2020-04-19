use sp_std::vec::Vec;
use crate::Node;

sp_api::decl_runtime_apis! {
    pub trait TeaApi {
        fn get_sum() -> u32;
        fn get_node(key: Vec<u8>) -> Node;
    }
}
