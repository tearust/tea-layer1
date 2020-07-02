// Tests to be written here

use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use hex::FromHex;
use std::vec::Vec;
use sp_core::{crypto, ed25519, hash::{H256, H512}, Pair};
use hex_literal::hex;

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
fn test_vector_should_work() {
	let pair = ed25519::Pair::from_seed(
		&hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
	);
	let public = pair.public();
	assert_eq!(public, ed25519::Public::from_raw(
		hex!("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
	));
	let message = b"";
	let signature = hex!("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b");
	let signature = ed25519::Signature::from_raw(signature);
	assert!(&pair.sign(&message[..]) == &signature);
	assert!(ed25519::Pair::verify(&signature, &message[..], &public));
}

#[test]
fn test_vector_by_string_should_work() {
	let pair = ed25519::Pair::from_string(
		"0x9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
		None
	).unwrap();
	let public = pair.public();
	assert_eq!(public, ed25519::Public::from_raw(
		hex!("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
	));
	let message = b"";
	let signature = hex!("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b");
	let signature = ed25519::Signature::from_raw(signature);
	assert!(&pair.sign(&message[..]) == &signature);
	assert!(ed25519::Pair::verify(&signature, &message[..], &public));
}
