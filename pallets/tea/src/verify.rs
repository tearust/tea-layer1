use sp_core::{crypto, crypto::{AccountId32, Public}, ed25519, sr25519, H256, U256, Pair};

pub fn verify_signature(public_key: [u8; 32], nonce_signature: Vec<u8>, data: Vec<u8> ) -> bool {
    let pubkey = ed25519::Public(public_key);
    let mut bytes = [0u8; 64];
    bytes.copy_from_slice(&nonce_signature);
    let signature = ed25519::Signature::from_raw(bytes);
    ed25519::Pair::verify(&signature, &data[..], &pubkey)
}