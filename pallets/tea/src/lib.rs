#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use frame_system::ensure_signed;
use codec::{Decode, Encode};
use frame_support::{
    debug,
    decl_event, decl_module, decl_storage, decl_error, dispatch,
    StorageMap, IterableStorageMap, ensure,
    traits::{Randomness, Currency, ExistenceRequirement,
             WithdrawReason, WithdrawReasons}};
use sp_std::prelude::*;
use sp_io::hashing::blake2_256;
use sp_core::{ed25519, U256};
use pallet_balances as balances;
use sp_runtime::traits::CheckedAdd;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub type TeaPubKey = [u8; 32];

pub type Url = Vec<u8>;

pub type Cid = Vec<u8>;

const RUNTIME_ACTIVITY_THRESHOLD: u32 = 3600;
// const MAX_RA_NODES_COUNT: u32 = 4;
const MIN_RA_PASSED_THRESHOLD: u32 = 3;

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

        // (rsa_pubkey, tea_id, block_number)
		Delegates get(fn delegates): Vec<(TeaPubKey, TeaPubKey, T::BlockNumber)>;
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
            delegator_pubkey: TeaPubKey,
		) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
		    ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);

            // Verify signature
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

            let mut exist = false;
            let current_block_number = <frame_system::Module<T>>::block_number();
            let mut delegates = Delegates::<T>::get();
            for (d, _t, b) in &mut delegates {
                if d == &delegator_pubkey {
                    *b = current_block_number;
                    exist = true;
                    break;
                }
            }
            if !exist {
                delegates.push((delegator_pubkey, tea_id, current_block_number));
            }
            Delegates::<T>::put(delegates);

            Self::deposit_event(RawEvent::UpdateRuntimeActivity(sender, runtime_activity));

            Ok(())
		}

		fn on_finalize(block_number: T::BlockNumber) {
            Self::update_runtime_status(block_number);
        }
	}
}

impl<T: Trait> Module<T> {
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

    // fn verify_tea_sig(tea_id: TeaPubKey,
    //                   tea_sig: Vec<u8>,
    //                   ephemeral_id: TeaPubKey) -> dispatch::DispatchResult {
    //     let tea_id = ed25519::Public(tea_id);
    //     ensure!(tea_sig.len() == 64, Error::<T>::InvalidTeaSig);
    //
    //     let tea_sig = ed25519::Signature::from_slice(&tea_sig[..]);
    //     ensure!(sp_io::crypto::ed25519_verify(&tea_sig, &ephemeral_id[..], &tea_id),
    //             Error::<T>::InvalidTeaSig);
    //
    //     Ok(())
    // }
    //
    // fn verify_delegate_sig(delegate_tea_id: TeaPubKey,
    //                        delegate_sig: Vec<u8>,
    //                        winner_tea_id: TeaPubKey,
    //                        ref_num: H256) -> dispatch::DispatchResult {
    //     let delegate_tea_id = ed25519::Public(delegate_tea_id);
    //     let auth_payload = [&winner_tea_id[..], &ref_num[..]].concat();
    //
    //     ensure!(delegate_sig.len() == 64, Error::<T>::InvalidSig);
    //     let delegate_sig = ed25519::Signature::from_slice(&delegate_sig[..]);
    //
    //     ensure!(sp_io::crypto::ed25519_verify(&delegate_sig, &auth_payload[..], &delegate_tea_id),
    //             Error::<T>::InvalidSig);
    //
    //     Ok(())
    // }
    //
    // fn verify_result_sig(executor_tea_id: TeaPubKey,
    //                      executor_sig: Vec<u8>,
    //                      result: &Vec<u8>) -> dispatch::DispatchResult {
    //     let executor_tea_id = ed25519::Public(executor_tea_id);
    //
    //     ensure!(executor_sig.len() == 64, Error::<T>::InvalidExecutorSig);
    //     let executor_sig = ed25519::Signature::from_slice(&executor_sig[..]);
    //
    //     ensure!(sp_io::crypto::ed25519_verify(&executor_sig, &result[..], &executor_tea_id),
    //             Error::<T>::InvalidExecutorSig);
    //
    //     Ok(())
    // }

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
}
