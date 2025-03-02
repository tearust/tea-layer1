// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

use hex as hex_o;
use hex_literal::hex;



use sp_core::{
    crypto, ed25519,
    hash::{H256, H512},
    sr25519, Pair,
};
use sp_runtime::traits::Verify;
use std::str::FromStr;
use std::vec::Vec;

#[test]
fn test_schnorr() {
    let message = b"We are legion!";
    let public_data: [u8; 32] = [
        122, 33, 114, 203, 102, 194, 85, 245, 50, 26, 127, 136, 99, 76, 50, 238, 21, 2, 127, 71,
        35, 62, 27, 178, 55, 118, 178, 46, 221, 206, 11, 93,
    ];
    let public = sr25519::Public::from_raw(public_data);

    let signature_data: [u8; 64] = [
        8, 79, 14, 237, 125, 143, 75, 146, 208, 134, 44, 92, 249, 82, 205, 23, 249, 20, 163, 223,
        28, 4, 203, 100, 24, 87, 206, 82, 53, 179, 243, 11, 81, 211, 221, 121, 187, 92, 211, 85,
        26, 76, 11, 177, 240, 58, 145, 40, 61, 4, 239, 254, 221, 182, 165, 211, 219, 23, 199, 40,
        110, 54, 62, 132,
    ];
    let signature = sr25519::Signature::from_raw(signature_data);

    println!("public: {:?}", public);
    println!("signature: {:?}", signature);
    assert!(sr25519::Pair::verify(&signature, &message[..], &public));
    assert!(signature.verify(&message[..], &public));
    assert!(sp_io::crypto::sr25519_verify(
        &signature,
        &message[..],
        &public
    ));
}

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        assert!(true);
        // asserting that the stored value is equal to what we stored
        // assert_eq!(TemplateModule::something(), Some(42));
    });
}

#[test]
fn test_vector_should_work() {
    let pair = ed25519::Pair::from_seed(&hex!(
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
    ));
    let public = pair.public();
    assert_eq!(
        public,
        ed25519::Public::from_raw(hex!(
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
        ))
    );
    let message = b"";
    let signature = hex!("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b");
    let signature = ed25519::Signature::from_raw(signature);
    assert!(&pair.sign(&message[..]) == &signature);
    assert!(ed25519::Pair::verify(&signature, &message[..], &public));
    assert!(signature.verify(&message[..], &public));
}

#[test]
fn test_vector_by_string_should_work() {
    let pair = ed25519::Pair::from_string(
        "0x9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
        None,
    )
    .unwrap();
    let public = pair.public();
    assert_eq!(
        public,
        ed25519::Public::from_raw(hex!(
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
        ))
    );
    let message = b"";
    let signature = hex!("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b");
    let signature = ed25519::Signature::from_raw(signature);
    assert!(&pair.sign(&message[..]) == &signature);
    assert!(ed25519::Pair::verify(&signature, &message[..], &public));
    assert!(signature.verify(&message[..], &public));
}

#[test]
fn test_ed25519_sig_should_work() {
    let public = ed25519::Public(hex!(
        "e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44"
    ));
    let message = hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a441d2905c84be11c0f314792f365e8385270495ebd112dc6363d3f02ec7ccfe475");
    let signature = hex!("26bcb7e99923c877cb6d50afedaf0fca0af4f3c78b437e8c48c7107f9ebdd1a00aa482e67ca244a40f44cf295d1b9f5c416202a5b785401408d8cffad0f18302");
    let signature = ed25519::Signature::from_raw(signature);
    assert!(ed25519::Pair::verify(&signature, &message[..], &public));
    assert!(signature.verify(&message[..], &public));
}

#[test]
fn test_auth_payload_should_right() {
    let winner_tea_id = hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
    let ref_num = hex!("0c6123c17c95bd6617a01ef899f5895ddb190eb3265f341687f4c0ad1b1f366f");

    let auth_payload = [&winner_tea_id[..], &ref_num[..]].concat();

    assert_eq!(hex_o::encode(auth_payload), "e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a440c6123c17c95bd6617a01ef899f5895ddb190eb3265f341687f4c0ad1b1f366f");
}

#[test]
fn test_delegate_sig_should_work() {
    let winner_tea_id = hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
    let ref_num = hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");

    let auth_payload = [&winner_tea_id[..], &ref_num[..]].concat();

    let public = ed25519::Public(hex!(
        "e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44"
    ));

    let signature = hex!("4f62e775a61ac904d6cfc18473203761b77ef60255c745ea71255606956596f0511ca61d1b2cd039d789f01a98f1d746bb6dc58feb07f945cc4c053050ab0103");
    let signature = ed25519::Signature::from_raw(signature);
    assert!(ed25519::Pair::verify(
        &signature,
        &auth_payload[..],
        &public
    ));
    assert!(signature.verify(&auth_payload[..], &public));
}

#[test]
fn get_tea_id_and_sig() {
    let pair = ed25519::Pair::from_seed(&hex!(
        "119c37b9aa65572ad9e24dd49c4f4da5330fe476f3313c560ffc67888f92b758"
    ));
    let public = pair.public();
    println!("pub: {:?}", public);

    let message = hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696597");
    let signature = &pair.sign(&message[..]);
    println!("sig: {:?}", signature);
}

#[test]
fn ed25519_sign_and_verify() {
    let pp = ed25519::Pair::from_string(&format!("//{}", "Bob"), None);
    let mut pair = ed25519::Pair::from_seed(&hex!(
        "119c37b9aa65572ad9e24dd49c4f4da5330fe476f3313c560ffc67888f92b758"
    ));
    match pp {
        Ok(p) => {
            pair = p;
        }
        Err(_e) => {
            println!("failed to parse account");
        }
    }
    let public = pair.public();
    println!("pub: {:?}", public);
    let message = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
    println!("message: {:?}", message);
    let signature = &pair.sign(&message[..]);
    assert!(signature.verify(&message[..], &public));
    println!("sig: {:?}", signature);
    assert!(ed25519::Pair::verify(&signature, &message[..], &public));

    let tea_id = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
    println!("tea_id:{:?}", tea_id);
    let ed25519_pubkey = ed25519::Public(hex!(
        "d17c2d7823ebf260fd138f2d7e27d114c0145d968b5ff5006125f2414fadae69"
    ));
    println!("publicKey:{:?}", ed25519_pubkey);
    let signature = hex!("36b3051088f0b81774c1f7c273cb95e626247a48a4a05d35ef31ca5f36bb548c4e064a4188d8fc806e452d2a2014b720dc70a60f70814e2b064e87f3df561e01");
    let signature = ed25519::Signature::from_raw(signature);
    println!("sig: {:?}", signature);
    assert!(ed25519::Pair::verify(
        &signature,
        &tea_id[..],
        &ed25519_pubkey
    ));
    assert!(signature.verify(&tea_id[..], &ed25519_pubkey));
}

#[test]
fn test_account_id_verification_should_work() {
    let message = hex!("1234");
    let signature = hex!("8ac5b79ca6ff412f576a0163b1daeb6b5d0a51fccab87b25c4d66d0323ca3a7ee95e4a036945409477bc9cd79f48e31c456ce6327ff1ba568817ce19cdee9e81");
    let signature = sr25519::Signature::from_raw(signature);

    let public =
        sr25519::Public::from_str("5GBykvvrUz3vwTttgHzUEPdm7G1FND1reBfddQLdiaCbhoMd").unwrap();
    println!("pub: {:?}", public);
    assert!(sr25519::Pair::verify(&signature, &message[..], &public));
    assert!(signature.verify(&message[..], &public));
}

#[test]
fn test_add_new_node() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        assert_ok!(TeaModule::add_new_node(Origin::signed(1), public));
        let target_node = Nodes::<Test>::get(&public).unwrap();
        assert_eq!(
            target_node.create_time,
            frame_system::Module::<Test>::block_number()
        );
    })
}

#[test]
fn test_add_new_node_already_exist() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        let _ = TeaModule::add_new_node(Origin::signed(1), public);

        assert_noop!(
            TeaModule::add_new_node(Origin::signed(1), public),
            Error::<Test>::NodeAlreadyExist
        );
    })
}

#[test]
fn test_update_manifest() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        let cid = vec![1, 2, 3];
        assert_ok!(TeaModule::update_manifest(
            Origin::signed(1),
            public,
            cid.clone()
        ));
        let target_cid = <Manifest>::get(&public).unwrap();
        assert_eq!(target_cid, cid);
    })
}

#[test]
fn test_remote_attestation_node_not_in_ra() {
    new_test_ext().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(100);

        let build_in_public =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let target_build_in_public =
            hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596");
        let ephemeral_public =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");

        let node = Node {
            tea_id: build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: Vec::new(),
            status: NodeStatus::Active,
        };
        Nodes::<Test>::insert(&build_in_public, node);
        BuildInNodes::insert(&build_in_public, &build_in_public);

        let is_pass = true;
        let signature = vec![1, 2, 3];
        assert_noop!(
            TeaModule::remote_attestation(
                Origin::signed(1),
                build_in_public,
                target_build_in_public,
                is_pass,
                signature
            ),
            Error::<Test>::NodeNotExist
        );
    })
}

#[test]
fn test_remote_attestation_node_not_exist() {
    new_test_ext().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(100);

        let build_in_public =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let target_build_in_public =
            hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596");
        let ephemeral_public =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");

        let node = Node {
            tea_id: build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: Vec::new(),
            status: NodeStatus::Active,
        };
        Nodes::<Test>::insert(&build_in_public, node);
        BuildInNodes::insert(&build_in_public, &build_in_public);

        let target_node = Node {
            tea_id: target_build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: [(build_in_public, false)].to_vec(),
            status: NodeStatus::Active,
        };
        Nodes::<Test>::insert(&target_build_in_public, target_node);
        BuildInNodes::insert(&target_build_in_public, &target_build_in_public);

        let is_pass = true;
        let signature = vec![1, 2, 3];
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        assert_noop!(
            TeaModule::remote_attestation(
                Origin::signed(1),
                build_in_public,
                public,
                is_pass,
                signature
            ),
            Error::<Test>::NodeNotExist
        );
    })
}

#[test]
fn test_remote_attestation_node_already_active() {
    new_test_ext().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(100);

        let build_in_public =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let target_build_in_public =
            hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596");
        let ephemeral_public =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");

        let node = Node {
            tea_id: build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: Vec::new(),
            status: NodeStatus::Active,
        };
        Nodes::<Test>::insert(&build_in_public, node);
        BuildInNodes::insert(&build_in_public, &build_in_public);

        let target_node = Node {
            tea_id: target_build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: [(build_in_public, false)].to_vec(),
            status: NodeStatus::Active,
        };
        Nodes::<Test>::insert(&target_build_in_public, target_node);
        BuildInNodes::insert(&target_build_in_public, &target_build_in_public);

        let is_pass = true;
        let signature = vec![1, 2, 3];
        assert_noop!(
            TeaModule::remote_attestation(
                Origin::signed(1),
                build_in_public,
                target_build_in_public,
                is_pass,
                signature
            ),
            Error::<Test>::NodeAlreadyActive
        );
    })
}

#[test]
fn test_remote_attestation() {
    new_test_ext().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(100);

        let build_in_public =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let target_build_in_public =
            hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596");
        let ephemeral_public =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");

        let node = Node {
            tea_id: build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: Vec::new(),
            status: NodeStatus::Active,
        };
        Nodes::<Test>::insert(&build_in_public, node);
        BuildInNodes::insert(&build_in_public, &build_in_public);

        let target_node = Node {
            tea_id: target_build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: [(build_in_public, false)].to_vec(),
            status: NodeStatus::Pending,
        };
        Nodes::<Test>::insert(&target_build_in_public, target_node);
        BuildInNodes::insert(&target_build_in_public, &target_build_in_public);

        let is_pass = true;
        let signature = vec![1, 2, 3];
        assert_ok!(TeaModule::remote_attestation(
            Origin::signed(1),
            build_in_public,
            target_build_in_public,
            is_pass,
            signature
        ));
    })
}

#[test]
fn test_update_node_profile() {
    new_test_ext().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(100);

        let build_in_public =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let ephemeral_public =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
        let profile_cid = Vec::new();
        let urls = Vec::new();
        let peer_id = Vec::new();
        let tea_sig = Vec::new();

        let node = Node {
            tea_id: build_in_public,
            ephemeral_id: ephemeral_public,
            profile_cid: Vec::new(),
            urls: Vec::new(),
            peer_id: Vec::new(),
            create_time: 0,
            update_time: 0,
            ra_nodes: Vec::new(),
            status: NodeStatus::Active,
        };
        Nodes::<Test>::insert(&build_in_public, node);
        BuildInNodes::insert(&build_in_public, &build_in_public);

        assert_ok!(TeaModule::update_node_profile(
            Origin::signed(1),
            build_in_public,
            ephemeral_public,
            profile_cid,
            urls,
            peer_id,
            tea_sig
        ));
    })
}

#[test]
fn test_update_node_profile_node_not_exist() {
    new_test_ext().execute_with(|| {
        let build_in_public =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let ephemeral_public =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
        let profile_cid = Vec::new();
        let urls = Vec::new();
        let peer_id = Vec::new();
        let tea_sig = Vec::new();

        for tea_id in BuildInNodes::iter() {
            println!("pub: {:?}", tea_id);
        }

        assert_noop!(
            TeaModule::update_node_profile(
                Origin::signed(1),
                build_in_public,
                ephemeral_public,
                profile_cid,
                urls,
                peer_id,
                tea_sig
            ),
            Error::<Test>::NodeNotExist
        );
    })
}

#[test]
fn test_add_new_service() {
    new_test_ext().execute_with(|| {
        let delegator_ephemeral_id =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let deployment_id = vec![1, 2, 3];
        let cid = vec![4, 5, 6];
        let cap_checker = vec![7, 8, 9];

        assert_ok!(TeaModule::add_new_service(
            Origin::signed(1),
            delegator_ephemeral_id,
            deployment_id.clone(),
            cid.clone(),
            cap_checker
        ));
        let service = ServiceMap::get(&cid).unwrap();
        assert_eq!(service.deployment_id, deployment_id);
    })
}

#[test]
fn test_deposit_amount_is_not_enough() {
    new_test_ext().execute_with(|| {
        let delegator_tea_id =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let delegator_ephemeral_id =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
        let delegator_signature = Vec::new();
        let amount: u64 = 100;

        assert_noop!(
            TeaModule::deposit(
                Origin::signed(1),
                delegator_tea_id,
                delegator_ephemeral_id,
                delegator_signature,
                amount,
                100
            ),
            pallet_balances::Error::<Test, _>::InsufficientBalance
        );
    })
}

#[test]
fn test_deposit() {
    new_test_ext().execute_with(|| {
        let delegator_tea_id =
            hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let delegator_ephemeral_id =
            hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
        let delegator_signature = Vec::new();
        let amount: u64 = 100;

        assert_ok!(TeaModule::deposit(
            Origin::signed(2),
            delegator_tea_id,
            delegator_ephemeral_id,
            delegator_signature,
            amount,
            100
        ));
    })
}

#[test]
fn test_settle_accounts() {
    new_test_ext().execute_with(|| {
        // origin,
        // use Lookup
        // employer: T::AccountId,
        // delegator_tea_id: TeaPubKey,
        // delegator_ephemeral_id: TeaPubKey,//+
        // errand_uuid: Vec<u8>,//-
        // errand_json_cid: Cid,//-
        // employer_sig: Vec<u8>,
        // executor_ephemeral_id: TeaPubKey,
        // expired_time: T::BlockNumber,
        // delegate_signature: Vec<u8>,
        // result_cid: Cid,
        // executor_singature: Vec<u8>,
        // bills: Vec<(T::AccountId, BalanceOf<T>)>,
        // todo complete me
    })
}

#[test]
fn test_update_runtime_activity() {
    new_test_ext().execute_with(|| {
        // origin,
        // tea_id: TeaPubKey,
        // cid: Cid,
        // ephemeral_id: TeaPubKey,
        // singature: Vec<u8>,
        // todo complete me
    })
}
