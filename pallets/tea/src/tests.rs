// Tests to be written here

use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use hex::FromHex;
use std::vec::Vec;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
        assert!(true);
		// asserting that the stored value is equal to what we stored
		// assert_eq!(TemplateModule::something(), Some(42));
	});
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct Node {
	tea_id: String,
	peers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
	account_id: String,
	delegate_node: Node,
	ref_num: u32,
	cap_cid: String,
	model_cid: String,
	data_cid: String,
	payment: u32,
}

#[test]
fn decode_task_msg() {
	let msg = r#"
		{
			"account_id":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
			"delegate_node":{"tea_id":"0x04","peers":["0x02","0x03"]},
			"ref_num":11,
			"cap_cid":"0x05",
			"model_cid":"0x06",
			"data_cid":"0x07",
			"payment":888
		}
	"#;
	let task: Task = serde_json::from_str(msg).unwrap();

	println!("{:#?}", task);

	assert_eq!(task.account_id, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
	assert_eq!(task.delegate_node, Node{tea_id: "0x04".into(), peers: vec!["0x02".into(), "0x03".into()]});
	assert_eq!(task.ref_num, 11);
	assert_eq!(task.cap_cid, "0x05");
	assert_eq!(task.model_cid, "0x06");
	assert_eq!(task.payment, 888);

	// hex string to vector sample
	let data_cid = Vec::from_hex(&task.data_cid[2..]).unwrap();
	assert_eq!(data_cid, vec![7]);
}