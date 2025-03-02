use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, crypto};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::AccountId32;
use tea_runtime::{
    AccountId, Balance, AuraConfig, BalancesConfig, GenesisConfig, GluonConfig, GrandpaConfig, Signature,
    SudoConfig, SystemConfig, TeaConfig, WASM_BINARY, AssetsConfig,
    ElectionsConfig, CouncilConfig, TechnicalCommitteeConfig,
};
use tea_runtime::constants::currency::*;

use std::str::FromStr;
use sp_core::crypto::Ss58Codec;

use hex_literal::hex;

use jsonrpc_core::serde_json;


// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

fn get_properties(symbol: &str) -> Properties {
    serde_json::json!({
        "tokenDecimals": 12,
        "ss58Format": 0,
        "tokenSymbol": symbol,
    }).as_object().unwrap().clone()
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Tea Layer1 Dev-Chain",
        // ID
        "layer1_dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    // authority_keys_from_seed("Dave"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    // get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(get_properties("TEA")),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    // get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    // get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}


/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    _endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {

    let FAUCET_ACCOUNT = crypto::AccountId32::from_str("5EtQMJ6mYtuzgtXiWCW8AjjxdHe4K3CUAWVkgU3agb2oKMGs").unwrap();

    let endowed_accounts: Vec<(AccountId, u128)> = {
		vec![
			(get_account_id_from_seed::<sr25519::Public>("Alice"), 10000*DOLLARS),
			(get_account_id_from_seed::<sr25519::Public>("Bob"), 100*DOLLARS),
			// (get_account_id_from_seed::<sr25519::Public>("Charlie"), 1000),
			// (get_account_id_from_seed::<sr25519::Public>("Dave"), 0),
			// (get_account_id_from_seed::<sr25519::Public>("Eve"), 0),
			// (get_account_id_from_seed::<sr25519::Public>("Ferdie"), 10000*DOLLARS),
			// get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			// get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			// get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			// get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			// get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
            // get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
            
            (FAUCET_ACCOUNT, 100000*DOLLARS)
		]
	};
	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
    const STASH: Balance = 100 * DOLLARS;
    
    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k.0, k.1))
                .collect(),
        }),
        pallet_aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        pallet_membership_Instance1: Default::default(),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key,
        }),

        pallet_elections_phragmen: Some(ElectionsConfig {
			members: vec![].iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.map(|member| (member, STASH))
						.collect(),
		}),
		pallet_collective_Instance1: Some(CouncilConfig::default()),
		pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
			members: vec![].iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			phantom: Default::default(),
		}),


        pallet_tea: Some(TeaConfig {
            tpms: vec![
                (
                    hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b"),
                    [0u8; 32],
                ),
                (
                    hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596"),
                    [0u8; 32],
                ),
                (
                    hex!("2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6"),
                    [0u8; 32],
                ),
                (
                    hex!("c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52"),
                    [0u8; 32],
                ),
                (
                    hex!("bd1c0ec25a96172791fe16c28323ceb0c515f17bcd11da4fb183ffd7e6fbb769"),
                    [0u8; 32],
                ),
            ],
        }),
        pallet_gluon: Some(GluonConfig {}),

        pallet_assets: Some(AssetsConfig {
            dai_list: vec![
                (get_account_id_from_seed::<sr25519::Public>("Alice"), 1389),
                (crypto::AccountId32::from_str("5EtQMJ6mYtuzgtXiWCW8AjjxdHe4K3CUAWVkgU3agb2oKMGs").unwrap(), 1389),
            ]
        }),
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1(){
        let address = "5EtQMJ6mYtuzgtXiWCW8AjjxdHe4K3CUAWVkgU3agb2oKMGs";

        let ac = crypto::AccountId32::from_str(address).unwrap();
        println!("{:?}", get_account_id_from_seed::<sr25519::Public>("Alice"));

        println!("{:#?}", ac);
    }
}