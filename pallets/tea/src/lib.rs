#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_system::ensure_signed;
use codec::{Decode, Encode};
use frame_support::{
    debug,
    decl_event, decl_module, decl_storage, decl_error, dispatch,
    StorageMap, StorageValue, IterableStorageMap, ensure,
    traits::{Randomness, Currency, ExistenceRequirement,  ExistenceRequirement::AllowDeath,
             WithdrawReason, WithdrawReasons, Imbalance}};
use sp_std::prelude::*;
use sp_io::hashing::blake2_256;
use sp_core::{crypto, crypto::{AccountId32, Public}, ed25519, sr25519, H256, U256};
use pallet_balances as balances;
use sp_runtime::traits::{
    Verify, IdentifyAccount, CheckedAdd, One, Zero,
};
use sha2::{Sha256, Digest};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "full_crypto")]
mod verify;

/// The pallet's configuration trait.
pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub type TeaPubKey = [u8; 32];

pub type Url = Vec<u8>;

pub type Cid = Vec<u8>;

pub type TxData = Vec<u8>;

pub type Signature = Vec<u8>;

pub type KeyType = Vec<u8>;

pub type ClientPubKey = Vec<u8>;

const RUNTIME_ACTIVITY_THRESHOLD: u32 = 3600;
const MAX_RA_NODES_COUNT: u32 = 4;
const MIN_RA_PASSED_THRESHOLD: u32 = 3;
const MIN_TRANSFER_ASSET_SIGNATURE_COUNT: usize = 2;
const MAX_TRANSFER_ASSET_TASK_PERIOD: u32 = 100;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum NodeStatus {
    Pending,
    Active,
    Inactive,
    Invalid,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// #[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
// #[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct Node<BlockNumber> {
    pub tea_id: TeaPubKey,
    pub ephemeral_id: TeaPubKey,
    pub profile_cid: Vec<u8>,
    pub urls: Vec<Url>,
    pub peer_id: Vec<u8>,
    pub create_time: BlockNumber,
    pub update_time: BlockNumber,
    pub ra_nodes: Vec<(TeaPubKey, bool)>,
    pub status: NodeStatus,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Deposit<Balance, BlockNumber> {
    /// Delegator device id.
    pub delegator_tea_id: TeaPubKey,
    /// Only this delegate node can grant an executor the errand.
    pub delegator_ephemeral_id: TeaPubKey,
    /// The delegator signature used to show that delegator has fulfilled its duties.
    pub delegator_signature: Vec<u8>,
    /// The deposit amount.
    pub amount: Balance,
    /// Specify the expiration height of the deposit.
    pub expire_time: BlockNumber,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Bill<AccountId, Balance, BlockNumber> {
    pub employer: AccountId,
    pub delegator_tea_id: TeaPubKey,
    pub delegator_ephemeral_id: TeaPubKey,
    pub errand_uuid: Vec<u8>,
    pub errand_json_cid: Cid,
    pub executor_ephemeral_id: TeaPubKey,
    pub expired_time: BlockNumber,
    pub result_cid: Cid,
    pub bills: Vec<(AccountId, Balance)>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Data {
    /// Others can look up layer1 to find out current peer_id
    pub delegator_ephemeral_id: TeaPubKey,
    /// It is the cid of pinner_key_pub
    pub deployment_id: Cid,
    /// Cid of encrypted data
    pub cid: Cid,
    /// Cid of description including price plan. How much to pay data owner per use
    pub description: Cid,
    /// Capability checker
    pub cap_checker: Cid,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Service {
    /// Others can look up layer1 to find out current peer_id
    pub delegator_ephemeral_id: TeaPubKey,
    /// It is the cid of pinner_key_pub
    pub deployment_id: Cid,
    /// Cid of encrypted data
    pub cid: Cid,
    /// Capability checker
    pub cap_checker: Cid,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct RaResult {
    pub tea_id: TeaPubKey,
    pub target_tea_id: TeaPubKey,
    pub is_pass: bool,
    pub target_status: NodeStatus,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ManifestInfo {
    pub tea_id: TeaPubKey,
    pub manifest_cid: Cid,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct RuntimeActivity<BlockNumber> {
    pub tea_id: TeaPubKey,
    pub cid: Cid,
    pub ephemeral_id: TeaPubKey,
    pub update_height: BlockNumber,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AccountAsset {
    pub account_id: Cid,
    pub btc: Vec<Cid>,
    pub eth: Vec<Cid>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct TransferAssetTask<BlockNumber> {
    pub from: Cid,
    pub to: Cid,
    pub start_height: BlockNumber,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct KeyGenerationData {
    /// the key type: btc or eth.
    pub key_type: Cid,
    /// split the secret to `n` pieces
    pub n: u32,
    /// if have k (k < n) pieces the secret can be recovered
    pub k: u32,
    /// tea id of delegator
    pub delegator_tea_id: TeaPubKey,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AccountGenerationData {
    /// the key type: btc or eth.
    pub key_type: Cid,
    /// split the secret to `n` pieces
    pub n: u32,
    /// if have k (k < n) pieces the secret can be recovered
    pub k: u32,
    /// tea id of delegator
    pub delegator_tea_id: TeaPubKey,
    /// p1 public key
    pub p1: Cid,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct KeyGenerationResult {
    pub task_id: Cid,
    pub public_key: Cid,
    pub deployment_ids: Vec<Cid>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct GluonWalletInfo {
    pub p2: Cid,
    pub deployment_ids: Vec<Cid>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct SignTransactionData {
    /// the task id of key generation
    pub key_task_id: Cid,
    /// the transaction to be signed
    pub data_adhoc: TxData,
    /// tea id of delegator
    pub delegator_tea_id: TeaPubKey,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct SignTransactionResult {
    pub task_id: Cid,
    pub signed_tx: TxData,
}

decl_storage! {
	trait Store for Module<T: Trait> as TeaModule {
	    Manifest get(fn manifest):
	        map hasher(twox_64_concat) TeaPubKey => Option<Cid>;

		Nodes get(fn nodes):
			map hasher(twox_64_concat) TeaPubKey => Option<Node<T::BlockNumber>>;
		EphemeralIds get(fn ephemera_ids):
		    map hasher(twox_64_concat) TeaPubKey => Option<TeaPubKey>;
        BootNodes get(fn boot_nodes):
            map hasher(twox_64_concat) TeaPubKey => TeaPubKey;
        BuildInNodes get(fn buildin_nodes):
            map hasher(twox_64_concat) TeaPubKey => Option<TeaPubKey>;
		PeerIds get(fn peer_ids):
		    map hasher(twox_64_concat) Vec<u8> => Option<TeaPubKey>;

		RuntimeActivities get(fn runtime_activities):
		    map hasher(twox_64_concat) TeaPubKey => Option<RuntimeActivity<T::BlockNumber>>;

		DataMap get(fn data_map):
		    map hasher(blake2_128_concat) Cid => Option<Data>;
		ServiceMap get(fn service_map):
		    map hasher(blake2_128_concat) Cid => Option<Service>;

		DepositMap get(fn deposit_map):
			map hasher(twox_64_concat) (T::AccountId, TeaPubKey) =>
			    Option<Deposit<BalanceOf<T>, T::BlockNumber>>;

	    TransferAssetTasks get(fn transfer_asset_tasks):
	        map hasher(twox_64_concat) Cid => TransferAssetTask<T::BlockNumber>;

	    TransferAssetSignatures get(fn transfer_asset_signatures):
	        map hasher(twox_64_concat) Cid => Vec<T::AccountId>;

	    AccountAssets get(fn account_assets):
	        map hasher(twox_64_concat) Cid => AccountAsset;

	    GenerateKeySenders get(fn generate_key_senders):
	        map hasher(blake2_128_concat) T::AccountId => Vec<Cid>;

	    GenerateKeyTasks get(fn generate_key_tasks):
	        map hasher(blake2_128_concat) Cid => KeyGenerationData;

	    GenerateKeyResults get(fn generate_key_results):
	        map hasher(blake2_128_concat) Cid => KeyGenerationResult;

	    SignTransactionTasks get(fn sign_transaction_tasks):
	        map hasher(blake2_128_concat) Cid => SignTransactionData;

	    SignTransactionResults get(fn sign_transaction_results):
	        map hasher(blake2_128_concat) Cid => SignTransactionResult;

        // Register Gluon wallet account.
        // Temporary storage
	    BrowserNonce get(fn browser_nonce):
	        map hasher(blake2_128_concat) T::AccountId => Cid;
	    // Permanent storage
        BrowserAppPair get(fn browser_app_pair):
            map hasher(blake2_128_concat) T::AccountId => T::AccountId;
        AppBrowserPair get(fn app_browser_pair):
            map hasher(blake2_128_concat) T::AccountId => T::AccountId;

        // Generate BTC 2/3 MultiSig Account
        // Temporary storage
        BrowserAccountNonce get(fn browser_account_nonce):
            map hasher(blake2_128_concat) T::AccountId => (Cid, Cid); // value: (nonce hash, task hash)
        // Intermediate storage to wait for task result
        AccountGenerationTaskDelegator get(fn account_generation_task_hashes):
            map hasher(blake2_128_concat) Cid => TeaPubKey; // key: app, value: key type
        // Permanent storage
        GluonWallets get(fn gluon_wallets):
            map hasher(blake2_128_concat) Cid => GluonWalletInfo; // key: multiSigAccount value
	}

	add_extra_genesis {
	    config(tpms): Vec<(TeaPubKey, TeaPubKey)>;
		build(|config: &GenesisConfig| {
			for (tea_id, ephemeral_id) in config.tpms.iter() {
				let node = Node {
				    tea_id: tea_id.clone(),
				    ephemeral_id: *ephemeral_id,
				    profile_cid: Vec::new(),
				    urls: Vec::new(),
				    peer_id: Vec::new(),
				    create_time: 0.into(),
				    update_time: 0.into(),
				    ra_nodes: Vec::new(),
				    status: NodeStatus::Active,
				};
				Nodes::<T>::insert(&tea_id, node);
				BuildInNodes::insert(&tea_id, &tea_id);
			}
		})
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId,
		Balance = BalanceOf<T>,
		BlockNumber = <T as frame_system::Trait>::BlockNumber,
	{
		NewNodeJoined(AccountId, Node<BlockNumber>),
		UpdateNodeProfile(AccountId, Node<BlockNumber>),
		NewDataAdded(AccountId, Data),
		NewServiceAdded(AccountId, Service),
		NewDepositAdded(AccountId, Deposit<Balance, BlockNumber>),
		SettleAccounts(AccountId, Bill<AccountId, Balance, BlockNumber>),
		CommitRaResult(AccountId, RaResult),
		UpdateManifest(AccountId, ManifestInfo),
		UpdateRuntimeActivity(AccountId, RuntimeActivity<BlockNumber>),
		TransferAssetBegin(Cid, TransferAssetTask<BlockNumber>),
		TransferAssetSign(Cid, AccountId),
		TransferAssetEnd(Cid, TransferAssetTask<BlockNumber>),
		KeyGenerationRequested(AccountId, Cid, KeyGenerationData),
		UpdateGenerateKey(Cid, KeyGenerationResult),
		SignTransactionRequested(AccountId, Cid, SignTransactionData),
		UpdateSignTransaction(Cid, SignTransactionResult),
		BrowserSendNonce(AccountId, Cid),
		RegistrationApplicationSucceed(AccountId, AccountId),
		BrowserAccountGeneration(AccountId, Cid, Cid),
		AccountGenrationRequested(AccountId, Cid, AccountGenerationData),
		GluonWalletGenerated(Cid, Cid, GluonWalletInfo),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
	    NodeAlreadyExist,
	    NodeNotExist,
	    InvalidSig,
	    InvalidExecutorSig,
	    InvalidTeaSig,
	    InvalidNonceSig,
	    InvalidExpairTime,
	    InvalidSignatureLength,
	    DelegatorNotExist,
	    InsufficientDeposit,
	    DepositAlreadyExist,
	    DepositNotExist,
	    PaymentOverflow,
	    NodeAlreadyActive,
	    NotInRaNodes,
        AccountIdConvertionError,
        InvalidToAccount,
        SenderIsNotBuildInAccount,
        SenderAlreadySigned,
        TransferAssetTaskTimeout,
        BrowserNonceAlreadyExist,
        AppBrowserPairAlreadyExist,
        NonceNotMatch,
        NonceNotExist,
        TaskNotMatch,
        TaskNotExist,
        KeyGenerationSenderAlreadyExist,
        KeyGenerationSenderNotExist,
        KeyGenerationTaskAlreadyExist,
        KeyGenerationResultExist,
        SignTransactionTaskAlreadyExist,
        SignTransactionResultExist,
        AccountGenerationTaskAlreadyExist,
        GluonWalletsAlreadyExist,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

        const TeaVersion: u32 = 5;

        #[weight = 100]
		pub fn add_new_node(origin, tea_id: TeaPubKey) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    ensure!(!Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeAlreadyExist);
		    let current_block_number = <frame_system::Module<T>>::block_number();

            let new_node = Node {
                tea_id: tea_id.clone(),
            	ephemeral_id: [0u8; 32],
            	profile_cid: Vec::new(),
            	urls: Vec::new(),
            	peer_id: Vec::new(),
            	create_time: current_block_number,
            	update_time: current_block_number,
            	ra_nodes: Vec::new(),
            	status: NodeStatus::Pending,
            };

            Nodes::<T>::insert(tea_id, &new_node);
            Self::deposit_event(RawEvent::NewNodeJoined(sender, new_node));

            Ok(())
		}

		#[weight = 100]
		pub fn update_manifest(origin, tea_id: TeaPubKey, manifest_cid: Cid) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;
            <Manifest>::insert(tea_id, &manifest_cid);

            let manifest_info = ManifestInfo {
                tea_id,
                manifest_cid,
            };
            Self::deposit_event(RawEvent::UpdateManifest(sender, manifest_info));

            Ok(())
		}

		#[weight = 100]
		pub fn remote_attestation(origin,
            tea_id: TeaPubKey,
            target_tea_id: TeaPubKey,
            is_pass: bool,
            signature: Vec<u8>,
		) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;

            // todo: verify signature

		    ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
		    ensure!(Nodes::<T>::contains_key(&target_tea_id), Error::<T>::NodeNotExist);
            let mut target_node = Nodes::<T>::get(&target_tea_id).unwrap();
            ensure!(target_node.status != NodeStatus::Active, Error::<T>::NodeAlreadyActive);

            // make sure tea_id in target ra_nodes vec.
            let mut exist_in_ra_nodes = false;
            let mut index = 0;
            for i in 0..target_node.ra_nodes.len() {
                index = i;
                let (ra_tea_id, _) = target_node.ra_nodes[i];
                if ra_tea_id == tea_id {
                    exist_in_ra_nodes = true;
                    break;
                }
            }
            ensure!(exist_in_ra_nodes, Error::<T>::NotInRaNodes);
            if is_pass {
                target_node.ra_nodes[index] = (tea_id, true);
                // calculate target node status
                let mut count = 0;
                for (_ra_tea_id, is_pass) in &target_node.ra_nodes {
                    if *is_pass {
                        count += 1;
                    }
                }
                // need 3/4 vote at least for now.
                if count >= MIN_RA_PASSED_THRESHOLD {
                    target_node.status = NodeStatus::Active;
                }
            } else {
                target_node.ra_nodes[index] = (tea_id, false);
                target_node.status = NodeStatus::Invalid;
            }
            Nodes::<T>::insert(target_tea_id, &target_node);

            let ra_result = RaResult {
                tea_id,
                target_tea_id,
                is_pass,
                target_status: target_node.status,
            };
            Self::deposit_event(RawEvent::CommitRaResult(sender, ra_result));

            Ok(())
		}

        #[weight = 100]
		pub fn update_node_profile(origin,
		    tea_id: TeaPubKey,
		    ephemeral_id: TeaPubKey,
		    profile_cid: Vec<u8>,
		    urls: Vec<Url>,
		    peer_id: Vec<u8>,
		    tea_sig: Vec<u8>,
		) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

		    ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
		    // Self::verify_tea_sig(tea_id.clone(), tea_sig, ephemeral_id)?;

            // remove old node info
            let old_node = Nodes::<T>::get(&tea_id).unwrap();
            BootNodes::remove(old_node.tea_id);
            EphemeralIds::remove(old_node.ephemeral_id);
            PeerIds::remove(old_node.peer_id);

            // generate ra nodes
            let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
            let payload = (
                random_seed,
                sender.clone(),
                tea_id.clone(),
                <frame_system::Module<T>>::block_number(),
            );
            let _random: U256 = payload.using_encoded(blake2_256).into();
            // todo: use random to generate ra nodes.
            // let index = random % 4;

            let mut ra_nodes = Vec::new();
            let mut status = NodeStatus::Pending;
            if BuildInNodes::get(tea_id).is_some() {
                // if tea_id is one of the build in nodes, set its status to active directly.
                status = NodeStatus::Active;
            } else {
                // select 4 build in nodes as ra nodes.
                for (tea_id, _) in BuildInNodes::iter() {
                    ra_nodes.push((tea_id, false));
                }
                // select 4 active nodes as ra nodes.
                // let mut count = 0;
                // for (tea_id, node) in Nodes::iter() {
                //     if node.status == NodeStatus::Active {
                //         ra_nodes.push((tea_id, false));
                //         count += 1;
                //     }
                //     if count == 4 {
                //         break;
                //     }
                // }
            }
            let current_block_number = <frame_system::Module<T>>::block_number();
		    let urls_count = urls.len();
            let node = Node {
                tea_id: tea_id.clone(),
            	ephemeral_id,
            	profile_cid,
            	urls,
            	peer_id: peer_id.clone(),
            	create_time: old_node.create_time,
            	update_time: current_block_number,
            	ra_nodes,
            	status,
            };
            Nodes::<T>::insert(&tea_id, &node);
	        EphemeralIds::insert(ephemeral_id, &tea_id);

            if urls_count > 0 {
                <BootNodes>::insert(&tea_id, &tea_id);
            }

            if peer_id.len() > 0 {
	            PeerIds::insert(peer_id, tea_id);
            }

            Self::deposit_event(RawEvent::UpdateNodeProfile(sender, node));

            Ok(())
		}

        #[weight = 100]
		pub fn add_new_data(
		    origin,
		    delegator_ephemeral_id: TeaPubKey,
		    deployment_id: Cid,
		    cid: Cid,
		    description: Cid,
		    cap_checker: Cid,
		    ) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;
		    // ensure!(!Models::<T>::contains_key(&cid), Error::<T>::ModelAlreadyExist);
            let new_data = Data {
                delegator_ephemeral_id,
                deployment_id,
                cid: cid.clone(),
                description,
                cap_checker,
            };
            DataMap::insert(cid, &new_data);
            Self::deposit_event(RawEvent::NewDataAdded(sender, new_data));

            Ok(())
		}

        #[weight = 100]
		pub fn add_new_service(
		    origin,
		    delegator_ephemeral_id: TeaPubKey,
		    deployment_id: Cid,
		    cid: Cid,
		    cap_checker: Cid,
		    ) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;
		    // ensure!(!Models::<T>::contains_key(&cid), Error::<T>::ModelAlreadyExist);
            let new_service = Service {
                delegator_ephemeral_id,
                deployment_id,
                cid: cid.clone(),
                cap_checker,
            };
            ServiceMap::insert(cid, &new_service);
            Self::deposit_event(RawEvent::NewServiceAdded(sender, new_service));

            Ok(())
		}

		#[weight = 100]
		pub fn deposit(
		    origin,
            delegator_tea_id: TeaPubKey,
            delegator_ephemeral_id: TeaPubKey,
            delegator_signature: Vec<u8>,
            amount: BalanceOf<T>,
            expire_time: T::BlockNumber,
		) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;
		    // todo: ensure delegator_tea_id exist

		    // todo: ensure delegator_ephemeral_id exist

		    // todo: ensure delegator_tea_id match delegator_ephemeral_id

		    // todo: ensure delegator_signature valid

		    // todo: ensure!(!DepositMap::<T>::contains_key((&sender, &delegator_tea_id)), Error::<T>::DepositAlreadyExist);

            let _neg_imbalance = T::Currency::withdraw(&sender,
		        amount,
		        WithdrawReasons::except(WithdrawReason::TransactionPayment),
		        ExistenceRequirement::AllowDeath)?;

            if DepositMap::<T>::contains_key((&sender, &delegator_tea_id)) {
                let mut deposit = DepositMap::<T>::get((&sender, &delegator_tea_id)).unwrap();
                // todo: verify if expire time GT old_expire_time + 100
		        // ensure!(expire_time > deposit.expire_time + 100, Error::<T>::InvalidExpairTime);
		        // ensure!(delegator_tea_id == deposit.delegator_tea_id, Error::<T>::InvalidSignatureLength);
                deposit.amount += amount;
                deposit.expire_time = expire_time;
                DepositMap::<T>::insert((&sender, &delegator_tea_id), &deposit);
                Self::deposit_event(RawEvent::NewDepositAdded(sender, deposit));
            } else {
                // todo: verify if expire time GT current block number + 100
                let new_deposit = Deposit {
                    delegator_tea_id,
                    delegator_ephemeral_id,
                    delegator_signature,
                    amount,
                    expire_time,
                };
                DepositMap::<T>::insert((&sender, &delegator_tea_id), &new_deposit);
                Self::deposit_event(RawEvent::NewDepositAdded(sender, new_deposit));
            }

            Ok(())
		}

		#[weight = 100]
		pub fn settle_accounts(
		    origin,
		    // use Lookup
            employer: T::AccountId,
            delegator_tea_id: TeaPubKey,
            delegator_ephemeral_id: TeaPubKey,//+
            errand_uuid: Vec<u8>,//-
            errand_json_cid: Cid,//-
            employer_sig: Vec<u8>,
            executor_ephemeral_id: TeaPubKey,
            expired_time: T::BlockNumber,
            delegate_signature: Vec<u8>,
            result_cid: Cid,
            executor_singature: Vec<u8>,
            bills: Vec<(T::AccountId, BalanceOf<T>)>,
		) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    // debug::info!("bill: {:?}", bill);
            // todo: check if the expired_time is lower than current block height, if false then treat the settle account
            //  as failed, and fire settle account request expired event

		    ensure!(DepositMap::<T>::contains_key((&employer, &delegator_tea_id)), Error::<T>::DepositNotExist);

            // todo: verify employer signature
            // let signer = employer.encode();
		    // let public = sr25519::Public::from_slice(&signer[..]);
            //
            // // ensure!(delegator_signature.len() == 64, Error::<T>::InvalidExecutorSig);
            // let signature = sr25519::Signature::from_slice(&delegator_signature[..]);
            // ensure!(signature.verify(&delegator_ephemeral_id[..], &public), Error::<T>::InvalidExecutorSig);

            // todo: limit bills array length
            let mut total_amount: BalanceOf<T> = BalanceOf::<T>::default();
            for (_account_id, payment) in &bills {
			    total_amount = total_amount.checked_add(payment).ok_or(Error::<T>::PaymentOverflow)?;
            }
            let mut deposit = DepositMap::<T>::get((&employer, &delegator_tea_id)).unwrap();
            ensure!(deposit.amount > total_amount, Error::<T>::InsufficientDeposit);
            deposit.amount -= total_amount;
            DepositMap::<T>::insert((&employer, &delegator_tea_id), deposit);

            debug::info!("deposit_creating total_amount: {:?}", total_amount);
            for (account_id, payment) in &bills {
                let _positive_imbalance = T::Currency::deposit_creating(account_id, *payment);
            }

            let bill = Bill {
                employer,
                delegator_tea_id,
                delegator_ephemeral_id,
                errand_uuid,
                errand_json_cid,
                executor_ephemeral_id,
                expired_time,
                result_cid,
                bills,
            };
            Self::deposit_event(RawEvent::SettleAccounts(sender, bill));

            Ok(())
		}

		#[weight = 100]
        pub fn update_runtime_activity(
		    origin,
            tea_id: TeaPubKey,
            cid: Cid,
            ephemeral_id: TeaPubKey,
            singature: Vec<u8>,
		) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
		    ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);

            // // Verify signature
            let ed25519_pubkey = ed25519::Public(ephemeral_id);
            let payload = [&tea_id[..], &cid[..]].concat();
            ensure!(singature.len() == 64, Error::<T>::InvalidSig);
            let ed25519_sig = ed25519::Signature::from_slice(&singature[..]);
            ensure!(sp_io::crypto::ed25519_verify(&ed25519_sig, &payload[..], &ed25519_pubkey),
                    Error::<T>::InvalidSig);

            let current_block_number = <frame_system::Module<T>>::block_number();
            let runtime_activity = RuntimeActivity {
                tea_id,
                cid,
                ephemeral_id,
                update_height: current_block_number,
            };

            RuntimeActivities::<T>::insert(&tea_id, &runtime_activity);
            Self::deposit_event(RawEvent::UpdateRuntimeActivity(sender, runtime_activity));

            Ok(())
		}

		#[weight = 100]
		pub fn send_nonce(
		    origin,
		    nonce_hash: Cid,
		) -> dispatch::DispatchResult {
            // todo add logic of timeout
		    let sender = ensure_signed(origin)?;
            ensure!(!BrowserNonce::<T>::contains_key(&sender), Error::<T>::BrowserNonceAlreadyExist);
            ensure!(!BrowserAppPair::<T>::contains_key(&sender), Error::<T>::AppBrowserPairAlreadyExist);

             // insert into BrowserNonce and fire an event
             BrowserNonce::<T>::insert(sender.clone(), nonce_hash.clone());
             Self::deposit_event(RawEvent::BrowserSendNonce(sender, nonce_hash));

            Ok(())
		}

		#[weight = 100]
		pub fn send_registration_application(
		    origin,
		    nonce: Cid,
            nonce_signature: Signature,
            browser_pk: ClientPubKey,
		) -> dispatch::DispatchResult {
            // todo add logic of timeout
            let sender = ensure_signed(origin)?;
            ensure!(!AppBrowserPair::<T>::contains_key(&sender), Error::<T>::AppBrowserPairAlreadyExist);
            ensure!(nonce_signature.len() == 64, Error::<T>::InvalidNonceSig);

            // check signature of appPubKey
            let mut app_pk = [0u8; 32];
            let app_public_key_bytes = Self::account_to_bytes(&sender);
            match app_public_key_bytes {
                Ok(p) => {
                    app_pk = p;
                }
                Err(_e) => {
                    debug::info!("failed to parse app account");
                    Err(Error::<T>::AccountIdConvertionError)?
                }
            }
            #[cfg(feature = "full_crypto")]
            ensure!(verify::verify_signature(app_pk, nonce_signature, nonce), Error::<T>::InvalidNonceSig);

            // check browser is not paired.
            let mut browser_account = T::AccountId::default();
            let browser = Self::bytes_to_account(&mut browser_pk.as_slice());
            match browser {
                Ok(p) => {
                    browser_account = p;
                }
                Err(_e) => {
                    debug::info!("failed to parse browser pubKey");
                    Err(Error::<T>::AccountIdConvertionError)?
                }
            }
            ensure!(!BrowserAppPair::<T>::contains_key(&browser_account), Error::<T>::AppBrowserPairAlreadyExist);
            ensure!(BrowserNonce::<T>::contains_key(&browser_account), Error::<T>::NonceNotExist);

            let app_nonce_hash = Self::sha2_256(&nonce.as_slice());
            let browser_nonce_hash = BrowserNonce::<T>::get(&browser_account);
            ensure!(browser_nonce_hash == app_nonce_hash, Error::<T>::NonceNotMatch);

            // pair succeed, record it into BrowserAppPair and AppBrowserPair and
            // remove data from AppRegistration and BrowserNonce.
            BrowserAppPair::<T>::insert(browser_account.clone(), sender.clone());
            AppBrowserPair::<T>::insert(browser_account.clone(), sender.clone());
            BrowserNonce::<T>::remove(browser_account.clone());

            // pair finished and fire an event
            Self::deposit_event(RawEvent::RegistrationApplicationSucceed(sender, browser_account));

            Ok(())
		}

		#[weight = 100]
		pub fn generate_key(
		    origin,
		    task_id: Cid,
		    delegator_tea_id: TeaPubKey,
		    key_type: Cid,
		    secret_pieces: u32,
		    sign_pieces: u32,
		) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;
            ensure!(!GenerateKeyTasks::contains_key(&task_id), Error::<T>::KeyGenerationTaskAlreadyExist);

            let task = KeyGenerationData {
                key_type: key_type.clone(),
                n: secret_pieces,
                k: sign_pieces,
                delegator_tea_id: delegator_tea_id,
            };
            if GenerateKeySenders::<T>::contains_key(&sender) {
                let mut task_ids = GenerateKeySenders::<T>::take(&sender);
                task_ids.push(task_id.clone());
                GenerateKeySenders::<T>::insert(&sender, task_ids);
            } else {
                GenerateKeySenders::<T>::insert(&sender, vec![task_id.clone()]);
            }
            GenerateKeyTasks::insert(task_id.clone(), task.clone());

            Self::deposit_event(RawEvent::KeyGenerationRequested(sender, task_id, task));

            Ok(())
		}

		#[weight = 100]
		pub fn update_generate_key_result(
		    origin,
		    task_id: Cid,
            public_key: Cid,
            deployment_ids: Vec<Cid>,
		) -> dispatch::DispatchResult {
		    let _sender = ensure_signed(origin)?;

            ensure!(GenerateKeyTasks::contains_key(&task_id), Error::<T>::TaskNotExist);
            ensure!(!GenerateKeyResults::contains_key(&task_id), Error::<T>::KeyGenerationResultExist);

            let key_generation_result = KeyGenerationResult {
                task_id: task_id.clone(),
                public_key: public_key.clone(),
                deployment_ids: deployment_ids.clone()
            };

            GenerateKeyResults::insert(task_id.clone(), key_generation_result.clone());
            Self::deposit_event(RawEvent::UpdateGenerateKey(task_id, key_generation_result));

            Ok(())
		}

        #[weight = 100]
        pub fn browser_generate_account(
            origin,
            nonce_hash: Cid,
            task_hash: Cid,
        ) -> dispatch::DispatchResult {
            // todo add logic of timeout
            let sender = ensure_signed(origin)?;
            ensure!(!BrowserAccountNonce::<T>::contains_key(&sender), Error::<T>::BrowserNonceAlreadyExist);

            // insert into BrowserNonce and fire an event
            BrowserAccountNonce::<T>::insert(sender.clone(), (nonce_hash.clone(), task_hash.clone()));
            Self::deposit_event(RawEvent::BrowserAccountGeneration(sender, nonce_hash, task_hash));

            Ok(())
        }

        #[weight = 100]
        pub fn generate_account_without_p3 (
           origin,
           nonce: Cid,
           nonce_signature: Cid,
           delegator_tea_id: TeaPubKey,
           key_type: Cid,
           p1: Cid,
           p2_n: u32,
           p2_k: u32,
           browser_pk: ClientPubKey,
        ) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(nonce_signature.len() == 64, Error::<T>::InvalidNonceSig);

            // check browser is not paired.
            let mut browser_account = T::AccountId::default();
            let browser = Self::bytes_to_account(&mut browser_pk.as_slice());
            match browser {
                Ok(p) => {
                    browser_account = p;
                }
                Err(_e) => {
                    debug::info!("failed to parse browser pubKey");
                    Err(Error::<T>::AccountIdConvertionError)?
                }
            }
            ensure!(BrowserAccountNonce::<T>::contains_key(&browser_account), Error::<T>::NonceNotExist);

            // check signature of appPubKey
            let mut app_pk = [0u8; 32];
            let app_public_key_bytes = Self::account_to_bytes(&sender);
            match app_public_key_bytes {
                Ok(p) => {
                    app_pk = p;
                }
                Err(_e) => {
                    debug::info!("failed to parse app account");
                    Err(Error::<T>::AccountIdConvertionError)?
                }
            }
            #[cfg(feature = "full_crypto")]
            ensure!(verify::verify_signature(app_pk, nonce_signature, nonce), Error::<T>::InvalidNonceSig);

            // check task hash
            let task = AccountGenerationData {
                key_type: key_type.clone(),
                n: p2_n,
                k: p2_k,
                delegator_tea_id: delegator_tea_id,
                p1: p1,
            };
            let task_data = task.encode();
            let task_hash = Self::sha2_256(&task_data);
            let app_nonce_hash = Self::sha2_256(&nonce.as_slice());
            let browser_nonce_hash = BrowserNonce::<T>::get(&browser_account);
            let (browser_nonce_hash, browser_task_hash) = BrowserAccountNonce::<T>::get(&browser_account);
            ensure!(browser_nonce_hash == app_nonce_hash, Error::<T>::NonceNotMatch);
            ensure!(browser_task_hash == task_hash, Error::<T>::TaskNotMatch);

            // account generation requested and fire an event
            AccountGenerationTaskDelegator::insert(task_hash.to_vec(), delegator_tea_id.clone());
            BrowserAccountNonce::<T>::remove(&browser_account);
            Self::deposit_event(RawEvent::AccountGenrationRequested(sender, task_hash.to_vec(), task));

            Ok(())
        }

		#[weight = 100]
        pub fn update_generate_account_without_p3_result(
	        origin,
	        task_id: Cid,
	        p2: Cid,
            p2_deployment_ids: Vec<Cid>,
            multiSigAccount: Cid,
        )-> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(AccountGenerationTaskDelegator::contains_key(&task_id), Error::<T>::TaskNotExist);
            ensure!(!GluonWallets::contains_key(&multiSigAccount), Error::<T>::GluonWalletsAlreadyExist);

            let mut delegator = [0u8; 32];
            let public_key_bytes = Self::account_to_bytes(&sender);
            match public_key_bytes {
                Ok(p) => {
                    delegator = p;
                }
                Err(_e) => {
                    debug::info!("failed to parse account");
                    Err(Error::<T>::AccountIdConvertionError)?
                }
            }
            let history_delegator = AccountGenerationTaskDelegator::get(&task_id);
            // todo if need check sender is delegator?
            ensure!(delegator == history_delegator, Error::<T>::InvalidSig);


            let gluon_wallet_info = GluonWalletInfo {
                p2: p2.clone(),
                deployment_ids: p2_deployment_ids.clone()
            };

            GluonWallets::insert(multiSigAccount.clone(), gluon_wallet_info.clone());
            Self::deposit_event(RawEvent::GluonWalletGenerated(task_id, multiSigAccount, gluon_wallet_info));

            Ok(())
        }

		#[weight = 100]
		pub fn sign_tx(
		    origin,
		    task_id: Cid,
		    key_task_id: Cid,
		    data_adhoc: TxData,
		    delegator_tea_id: TeaPubKey,
		) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

            ensure!(GenerateKeySenders::<T>::contains_key(&sender), Error::<T>::KeyGenerationSenderNotExist);
            ensure!(!SignTransactionTasks::contains_key(&task_id), Error::<T>::SignTransactionTaskAlreadyExist);

            let task = SignTransactionData {
                key_task_id: key_task_id,
                data_adhoc: data_adhoc,
                delegator_tea_id: delegator_tea_id,
            };
            SignTransactionTasks::insert(task_id.clone(), task.clone());

            Self::deposit_event(RawEvent::SignTransactionRequested(sender, task_id, task));

            Ok(())
		}

		#[weight = 100]
		pub fn update_sign_tx_result(
		    origin,
		    task_id: Cid,
            signed_tx: TxData,
		) -> dispatch::DispatchResult {
		    let _sender = ensure_signed(origin)?;

            ensure!(SignTransactionTasks::contains_key(&task_id), Error::<T>::TaskNotExist);
            ensure!(!SignTransactionResults::contains_key(&task_id), Error::<T>::SignTransactionResultExist);

            let sign_transaction_result = SignTransactionResult {
                task_id: task_id.clone(),
                signed_tx: signed_tx,
            };

            SignTransactionResults::insert(task_id.clone(), sign_transaction_result.clone());
            Self::deposit_event(RawEvent::UpdateSignTransaction(task_id, sign_transaction_result));

            Ok(())
		}

		#[weight = 100]
        pub fn transfer_asset(
		    origin,
            from: Cid,
            to: Cid,
		) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;
		    ensure!(from != to, Error::<T>::InvalidToAccount);
		    let current_block_number = <frame_system::Module<T>>::block_number();
		    if TransferAssetTasks::<T>::contains_key(&from) {
		        ensure!(TransferAssetTasks::<T>::get(&from).to == to, Error::<T>::InvalidToAccount);
		        // if timeout, need to remove the transfer-asset task.
		        if TransferAssetTasks::<T>::get(&from).start_height +
		        MAX_TRANSFER_ASSET_TASK_PERIOD.into() < current_block_number {
		            TransferAssetTasks::<T>::remove(&from);
		            TransferAssetSignatures::<T>::remove(&from);
		        }
		    }

            let mut client_from = T::AccountId::default();
            let account_from = Self::bytes_to_account(&mut from.as_slice());
            match account_from {
                Ok(f) => {
                    client_from = f;
                }
                Err(_e) => {
                    debug::info!("failed to parse client");
                    Err(Error::<T>::AccountIdConvertionError)?
                }
            }
            let mut client_to = T::AccountId::default();
            let account_to = Self::bytes_to_account(&mut from.as_slice());
            match account_to {
                Ok(t) => {
                    client_to = t;
                }
                Err(_e) => {
                    debug::info!("failed to parse client");
                    Err(Error::<T>::AccountIdConvertionError)?
                }
            }

            let account_vec = sender.encode();
            ensure!(account_vec.len() == 32, Error::<T>::AccountIdConvertionError);
            let mut tea_id = [0u8; 32];
            tea_id.copy_from_slice(&account_vec);
            ensure!(BuildInNodes::get(tea_id).is_some(), Error::<T>::SenderIsNotBuildInAccount);

             if TransferAssetSignatures::<T>::contains_key(&from) {
                 let signatures = TransferAssetSignatures::<T>::get(&from);
                 for sig in signatures.iter() {
                    ensure!(&sender != sig, Error::<T>::SenderAlreadySigned);
                 }
                 let mut signatures = TransferAssetSignatures::<T>::take(&from);
                 signatures.push(sender.clone());
                 TransferAssetSignatures::<T>::insert(&from, signatures.clone());
                 Self::deposit_event(RawEvent::TransferAssetSign(from.clone(), sender.clone()));

                 if signatures.len() >= MIN_TRANSFER_ASSET_SIGNATURE_COUNT {
                    // transfer balance
                    let total_balance = T::Currency::total_balance(&client_from);
                    if total_balance > 0.into() {
                        T::Currency::transfer(&client_from, &client_to, total_balance, AllowDeath)?;
                        TransferAssetSignatures::<T>::remove(&from);
                        TransferAssetTasks::<T>::remove(&from);
                    }

                    // todo transfer btc and other asset
                    if AccountAssets::contains_key(&from) {
                        let mut from_account_assets = AccountAssets::take(&from);
                        if AccountAssets::contains_key(&to) {
                            let mut to_account_assets = AccountAssets::take(&to);
                            to_account_assets.btc.append(&mut from_account_assets.btc);
                            to_account_assets.eth.append(&mut from_account_assets.eth);
                            AccountAssets::insert(&to, to_account_assets);
                        } else {
                            AccountAssets::insert(&to, from_account_assets);
                        }

                    }

                    Self::deposit_event(RawEvent::TransferAssetEnd(from.clone(), TransferAssetTasks::<T>::get(&from)));
                 }
             } else {
                 TransferAssetSignatures::<T>::insert(&from, vec![sender.clone()]);
             }

             if !TransferAssetTasks::<T>::contains_key(&from) {
                let new_task = TransferAssetTask {
                    from: from.clone(),
            	    to: to.clone(),
            	    start_height: current_block_number,
                };
                TransferAssetTasks::<T>::insert(from.clone(), &new_task);
                Self::deposit_event(RawEvent::TransferAssetBegin(from.clone(), new_task));
             }

             Ok(())
		}

		fn on_finalize(block_number: T::BlockNumber) {
            Self::update_runtime_status(block_number);
        }
	}
}

impl<T: Trait> Module<T> {
    fn bytes_to_account(mut account_bytes: &[u8]) -> Result<T::AccountId, Error<T>> {
        match T::AccountId::decode(&mut account_bytes) {
            Ok(client) => {
                return Ok(client);
            }
            Err(_e) => Err(Error::<T>::AccountIdConvertionError),
        }
    }

    fn account_to_bytes(account: &T::AccountId) -> Result<[u8; 32], Error<T>> {
        let account_vec = account.encode();
        if account_vec.len() != 32 {
            return Err(Error::<T>::AccountIdConvertionError);
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        Ok(bytes)
    }

    fn update_runtime_status(block_number: T::BlockNumber) {
        for (tea_id, mut node) in Nodes::<T>::iter() {
            if node.status == NodeStatus::Active {
                if block_number - node.update_time <= RUNTIME_ACTIVITY_THRESHOLD.into() {
                    continue;
                }
                match RuntimeActivities::<T>::get(&tea_id) {
                    Some(runtime_activity) => {
                        if block_number - runtime_activity.update_height > RUNTIME_ACTIVITY_THRESHOLD.into() {
                            node.status = NodeStatus::Inactive;
                            Nodes::<T>::insert(&tea_id, node);
                        }
                    }
                    None => {
                        node.status = NodeStatus::Inactive;
                        Nodes::<T>::insert(&tea_id, node);
                    }
                }
            }
        }
    }

    fn verify_tea_sig(tea_id: TeaPubKey,
                      tea_sig: Vec<u8>,
                      ephemeral_id: TeaPubKey) -> dispatch::DispatchResult {
        let tea_id = ed25519::Public(tea_id);
        ensure!(tea_sig.len() == 64, Error::<T>::InvalidTeaSig);

        let tea_sig = ed25519::Signature::from_slice(&tea_sig[..]);
        ensure!(sp_io::crypto::ed25519_verify(&tea_sig, &ephemeral_id[..], &tea_id),
                Error::<T>::InvalidTeaSig);

        Ok(())
    }

    fn verify_delegate_sig(delegate_tea_id: TeaPubKey,
                           delegate_sig: Vec<u8>,
                           winner_tea_id: TeaPubKey,
                           ref_num: H256) -> dispatch::DispatchResult {
        let delegate_tea_id = ed25519::Public(delegate_tea_id);
        let auth_payload = [&winner_tea_id[..], &ref_num[..]].concat();

        ensure!(delegate_sig.len() == 64, Error::<T>::InvalidSig);
        let delegate_sig = ed25519::Signature::from_slice(&delegate_sig[..]);

        ensure!(sp_io::crypto::ed25519_verify(&delegate_sig, &auth_payload[..], &delegate_tea_id),
                Error::<T>::InvalidSig);

        Ok(())
    }

    fn verify_result_sig(executor_tea_id: TeaPubKey,
                         executor_sig: Vec<u8>,
                         result: &Vec<u8>) -> dispatch::DispatchResult {
        let executor_tea_id = ed25519::Public(executor_tea_id);

        ensure!(executor_sig.len() == 64, Error::<T>::InvalidExecutorSig);
        let executor_sig = ed25519::Signature::from_slice(&executor_sig[..]);

        ensure!(sp_io::crypto::ed25519_verify(&executor_sig, &result[..], &executor_tea_id),
                Error::<T>::InvalidExecutorSig);

        Ok(())
    }

    // JSON-RPC implementation.
    pub fn get_sum() -> u32 {
        100 + 200
    }

    pub fn get_node_by_ephemeral_id(ephemeral_id: TeaPubKey) -> Option<Node<T::BlockNumber>> {
        let tea_id = Self::ephemera_ids(ephemeral_id);
        debug::info!("get_node_by_ephemeral_id(): {:?}", tea_id);

        return match tea_id {
            Some(id) => {
                Self::nodes(id)
            }
            None => None
        };
    }

    /// Do a sha2 256-bit hash and return result.
    pub fn sha2_256(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.input(data);
        let mut output = [0u8; 32];
        output.copy_from_slice(&hasher.result());
        output
    }
}
