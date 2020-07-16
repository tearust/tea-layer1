use sp_std::vec::Vec;
use crate::Node;
use crate::TeaPubKey;

sp_api::decl_runtime_apis! {
    pub trait TeaApi {
        fn get_sum() -> u32;
        fn get_node_by_ephemeral_id(id: TeaPubKey) -> Option<Node>;
    }
}