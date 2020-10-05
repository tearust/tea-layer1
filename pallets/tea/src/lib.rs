#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use system::ensure_signed;
use codec::{Decode, Encode};
use frame_support::{
    debug,
    decl_event, decl_module, decl_storage, decl_error, dispatch,
    StorageMap, StorageValue, IterableStorageMap, ensure,
    traits::{Randomness, Currency, ExistenceRequirement, WithdrawReason, WithdrawReasons,
             Imbalance}};
use sp_std::prelude::*;
use sp_io::hashing::blake2_256;
use sp_core::{crypto::{AccountId32,Public}, ed25519, sr25519, H256, U256};
use pallet_balances as balances;
use sp_runtime::traits::{
        Verify,IdentifyAccount,CheckedAdd,One,Zero,
    };

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod api;

/// The pallet's configuration trait.
pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub type TeaPubKey = [u8; 32];

type Url = Vec<u8>;

type Cid = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
enum NodeStatus {
    Pending,
    Active,
    Invalid,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// #[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
// #[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct Node {
    tea_id: TeaPubKey,
    ephemeral_id: TeaPubKey,
    profile_cid: Vec<u8>,
    urls: Vec<Url>,
    peer_id: Vec<u8>,
    create_time: u64,
    ra_nodes: Vec<(TeaPubKey, bool)>,
    status: NodeStatus,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Deposit<Balance> {
    /// Delegator device id.
    delegator_tea_id: TeaPubKey,
    /// Only this delegate node can grant an executor the errand.
    delegator_ephemeral_id: TeaPubKey,
    /// The delegator signature used to show that delegator has fulfilled its duties.
    delegator_signature: Vec<u8>,
    /// The deposit amount.
    amount: Balance,
    /// Specify the expiration height of the deposit.
    expire_time: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Bill<AccountId, Balance> {
    employer: AccountId,
    delegator_tea_id: TeaPubKey,
    delegator_ephemeral_id: TeaPubKey,
    errand_uuid: Vec<u8>,
    errand_json_cid: Cid,
    executor_ephemeral_id: TeaPubKey,
    expired_time: u64,
    result_cid: Cid,
    bills: Vec<(AccountId, Balance)>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Data {
    /// Others can look up layer1 to find out current peer_id
    delegator_ephemeral_id: TeaPubKey,
    /// It is the cid of pinner_key_pub
    deployment_id: Cid,
    /// Cid of encrypted data
    cid: Cid,
    /// Cid of description including price plan. How much to pay data owner per use
    description: Cid,
    /// Capability checker
    cap_checker: Cid,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Service {
    /// Others can look up layer1 to find out current peer_id
    delegator_ephemeral_id: TeaPubKey,
    /// It is the cid of pinner_key_pub
    deployment_id: Cid,
    /// Cid of encrypted data
    cid: Cid,
    /// Capability checker
    cap_checker: Cid,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct RaResult {
    tea_id: TeaPubKey,
    target_tea_id: TeaPubKey,
    is_pass: bool,
    target_status: NodeStatus,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ManifestInfo {
    tea_id: TeaPubKey,
    manifest_cid: Cid,
}

decl_storage! {
	trait Store for Module<T: Trait> as TeaModule {
	    Manifest get(fn manifest):
	        map hasher(twox_64_concat) TeaPubKey => Option<Cid>;

		Nodes get(fn nodes):
			map hasher(twox_64_concat) TeaPubKey => Option<Node>;
		EphemeralIds get(fn ephemera_ids):
		    map hasher(twox_64_concat) TeaPubKey => Option<TeaPubKey>;
        BootNodes get(fn boot_nodes):
            map hasher(twox_64_concat) TeaPubKey => TeaPubKey;
		PeerIds get(fn peer_ids):
		    map hasher(twox_64_concat) Vec<u8> => Option<TeaPubKey>;

		DataMap get(fn data_map):
		    map hasher(blake2_128_concat) Cid => Option<Data>;
		ServiceMap get(fn service_map):
		    map hasher(blake2_128_concat) Cid => Option<Service>;

		DepositMap get(fn deposit_map):
			map hasher(twox_64_concat) (T::AccountId, TeaPubKey) => Option<Deposit<BalanceOf<T>>>;
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
				    create_time: 0,
				    ra_nodes: Vec::new(),
				    status: NodeStatus::Active,
				};
				Nodes::insert(tea_id, node);
			}
		})
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
	{
		NewNodeJoined(AccountId, Node),
		UpdateNodeProfile(AccountId, Node),
		NewDataAdded(AccountId, Data),
		NewServiceAdded(AccountId, Service),
		NewDepositAdded(AccountId, Deposit<Balance>),
		SettleAccounts(AccountId, Bill<AccountId, Balance>),
		CommitRaResult(AccountId, RaResult),
		UpdateManifest(AccountId, ManifestInfo),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
	    NodeAlreadyExist,
	    NodeNotExist,
	    InvalidDelegateSig,
	    InvalidExecutorSig,
	    InvalidTeaSig,
	    InvalidExpairTime,
	    InvalidSignatureLength,
	    DelegatorNotExist,
	    InsufficientDeposit,
	    DepositAlreadyExist,
	    DepositNotExist,
	    PaymentOverflow,
	    NodeAlreadyActive,
	    NotInRaNodes,
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

        #[weight = 100]
		pub fn add_new_node(origin, tea_id: TeaPubKey) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    ensure!(!Nodes::contains_key(&tea_id), Error::<T>::NodeAlreadyExist);

            let new_node = Node {
                tea_id: tea_id.clone(),
            	ephemeral_id: [0u8; 32],
            	profile_cid: Vec::new(),
            	urls: Vec::new(),
            	peer_id: Vec::new(),
            	create_time: 0,
            	ra_nodes: Vec::new(),
            	status: NodeStatus::Pending,
            };

            <Nodes>::insert(tea_id, &new_node);
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

		    ensure!(Nodes::contains_key(&tea_id), Error::<T>::NodeNotExist);
		    ensure!(Nodes::contains_key(&target_tea_id), Error::<T>::NodeNotExist);
            let mut target_node = Nodes::get(&target_tea_id).unwrap();
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
                // need 2/4 vote at least.
                if count > 1 {
                    target_node.status = NodeStatus::Active;
                }
            } else {
                target_node.ra_nodes[index] = (tea_id, false);
                target_node.status = NodeStatus::Invalid;
            }
            Nodes::insert(target_tea_id, &target_node);

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

		    ensure!(Nodes::contains_key(&tea_id), Error::<T>::NodeNotExist);
		    // Self::verify_tea_sig(tea_id.clone(), tea_sig, ephemeral_id)?;

            // remove old node info
            let old_node = Nodes::get(&tea_id).unwrap();
            BootNodes::remove(old_node.tea_id);
            EphemeralIds::remove(old_node.ephemeral_id);
            PeerIds::remove(old_node.peer_id);

            // generate ra nodes
            let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
            let payload = (
                random_seed,
                sender.clone(),
                tea_id.clone(),
                <system::Module<T>>::block_number(),
            );
            let _random: U256 = payload.using_encoded(blake2_256).into();

            // todo: use random to generate ra nodes.
            // let index = random % 4;

            // select first 4 nodes as ra nodes for dev.
            // todo: after register bootstrap nodes on layer1, and judge if it is bootstrap node
            //      updating node profile here, if true then shall have no ra nodes
            let mut count = 0;
            let mut ra_nodes = Vec::new();
            for (tea_id, node) in Nodes::iter() {
                if node.status == NodeStatus::Active {
                    ra_nodes.push((tea_id, false));
                    count += 1;
                }
                if count == 4 {
                    break;
                }
            }

		    let urls_count = urls.len();
            let node = Node {
                tea_id: tea_id.clone(),
            	ephemeral_id,
            	profile_cid,
            	urls,
            	peer_id: peer_id.clone(),
            	create_time: old_node.create_time,
            	ra_nodes: ra_nodes,
            	status: old_node.status,
            };
            <Nodes>::insert(&tea_id, &node);
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
            expire_time: u64,
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
            expired_time: u64,
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
	}
}

impl<T: Trait> Module<T> {
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

        ensure!(delegate_sig.len() == 64, Error::<T>::InvalidDelegateSig);
        let delegate_sig = ed25519::Signature::from_slice(&delegate_sig[..]);

        ensure!(sp_io::crypto::ed25519_verify(&delegate_sig, &auth_payload[..], &delegate_tea_id),
                Error::<T>::InvalidDelegateSig);

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

    pub fn get_node_by_ephemeral_id(ephemeral_id: TeaPubKey) -> Option<Node> {
        let tea_id = Self::ephemera_ids(ephemeral_id);
        debug::info!("get_node_by_ephemeral_id(): {:?}", tea_id);

        return match tea_id {
            Some(id) => {
                Self::nodes(id)
            },
            None => None
        }
    }
}
