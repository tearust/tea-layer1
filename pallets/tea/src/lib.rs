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
    StorageMap, StorageValue, ensure,
    traits::{Randomness, Currency, ExistenceRequirement, WithdrawReason, WithdrawReasons,
             Imbalance}};
use sp_std::prelude::*;
use sp_io::hashing::blake2_256;
use sp_core::{crypto::{AccountId32}, ed25519, sr25519, hash::{H256}};
use pallet_balances as balances;
use sp_runtime::traits::{Verify,IdentifyAccount};

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

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct Node {
    tea_id: TeaPubKey,
    ephemeral_id: TeaPubKey,
    profile_cid: Vec<u8>,
    urls: Vec<Url>,
    peer_id: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Model<AccountId> {
    account: AccountId,
    price: u32,
    cid: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Deposit<Balance> {
    /// Delegator device id.
    // delegator_tea_id: TeaPubKey,
    /// Only this delegate node can grant an executor the errand.
    delegator_ephemeral_id: TeaPubKey,
    /// An ed25519 public key use for grant a delegate node the errand.
    deposit_pub_key: TeaPubKey,
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
    delegator_ephemeral_id: TeaPubKey,
    errand_uuid: Vec<u8>,
    payment: Balance,
    payment_type: u32,
    executor_ephemeral_id: TeaPubKey,
    expired_time: u64,
    result_cid: Cid,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Task<Balance> {
    ref_num: H256,
    delegate_tea_id: TeaPubKey,
    model_cid: Vec<u8>,
    body_cid: Vec<u8>,
    payment: Balance,
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

decl_storage! {
	trait Store for Module<T: Trait> as TeaModule {
		Nodes get(fn nodes):
			map hasher(twox_64_concat) TeaPubKey => Option<Node>;
		EphemeralIds get(fn ephemera_ids):
		    map hasher(twox_64_concat) TeaPubKey => Option<TeaPubKey>;
        BootNodes get(fn boot_nodes):
            map hasher(twox_64_concat) TeaPubKey => TeaPubKey;
		PeerIds get(fn peer_ids):
		    map hasher(twox_64_concat) Vec<u8> => Option<TeaPubKey>;

		Models get(fn models):
			map hasher(blake2_128_concat) Vec<u8> => Model<T::AccountId>;
		Tasks get(fn tasks):
			map hasher(blake2_128_concat) H256 => Option<Task<BalanceOf<T>>>;

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
		RefNum = H256,
		Result = Vec<u8>,
	{
		NewNodeJoined(AccountId, Node),
		UpdateNodeProfile(AccountId, Node),
		NewModelAdded(AccountId),
		NewTaskAdded(AccountId, Task<Balance>),
		CompleteTask(AccountId, RefNum, Result),
		NewDataAdded(AccountId, Data),
		NewServiceAdded(AccountId, Service),
		NewDepositAdded(AccountId, Deposit<Balance>),
		SettleAccounts(AccountId, Bill<AccountId, Balance>),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
	    NodeAlreadyExist,
	    NodeNotExist,
	    ModelAlreadyExist,
	    ModelNotExist,
	    TaskNotExist,
	    TaskCountOverflow,
	    InvalidDelegateSig,
	    InvalidExecutorSig,
	    InvalidTeaSig,
	    InvalidExpairTime,
	    InvalidDepositPubkey,
	    DelegatorNotExist,
	    InsufficientDeposit,
	    DepositAlreadyExist,
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

        #[weight = 0]
		pub fn add_new_node(origin, tea_id: TeaPubKey) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    ensure!(!Nodes::contains_key(&tea_id), Error::<T>::NodeAlreadyExist);

            let new_node = Node {
                tea_id: tea_id.clone(),
            	ephemeral_id: [0u8; 32],
            	profile_cid: Vec::new(),
            	urls: Vec::new(),
            	peer_id: Vec::new(),
            };
            <Nodes>::insert(tea_id, &new_node);
            Self::deposit_event(RawEvent::NewNodeJoined(sender, new_node));

            Ok(())
		}

        #[weight = 0]
		pub fn update_node_profile(origin,
		    tea_id: TeaPubKey,
		    ephemeral_id: TeaPubKey,
		    profile_cid: Vec<u8>,
		    urls: Vec<Url>,
		    peer_id: Vec<u8>,
		    tea_sig: Vec<u8>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

		    ensure!(Nodes::contains_key(&tea_id), Error::<T>::NodeNotExist);
		    Self::verify_tea_sig(tea_id.clone(), tea_sig, ephemeral_id)?;

            // remove old node info
            let old_node = Nodes::get(&tea_id).unwrap();
            BootNodes::remove(old_node.tea_id);
            EphemeralIds::remove(old_node.ephemeral_id);
            PeerIds::remove(old_node.peer_id);

		    let urls_count = urls.len();
            let node = Node {
                tea_id: tea_id.clone(),
            	ephemeral_id,
            	profile_cid,
            	urls,
            	peer_id: peer_id.clone(),
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

        #[weight = 0]
		pub fn add_new_model(origin, price: u32, cid: Vec<u8>) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    ensure!(!Models::<T>::contains_key(&cid), Error::<T>::ModelAlreadyExist);

            let new_model = Model {
                account: sender.clone(),
                price,
                cid: cid.clone(),
            };
            <Models<T>>::insert(cid, new_model);
            Self::deposit_event(RawEvent::NewModelAdded(sender));

            Ok(())
		}

        #[weight = 0]
		pub fn add_new_task(origin,
		    ref_num: H256,
		    delegate_tea_id: TeaPubKey,
		    model_cid: Vec<u8>,
		    body_cid: Vec<u8>,
		    payment: BalanceOf<T>)
		{
			let sender = ensure_signed(origin)?;

		    // ensure!(Nodes::contains_key(&delegate_tea_id), Error::<T>::NodeNotExist);
		    // let node = Nodes::get(&delegate_tea_id).unwrap();

            let neg_imbalance = T::Currency::withdraw(&sender,
		        payment,
		        WithdrawReasons::except(WithdrawReason::TransactionPayment),
		        ExistenceRequirement::AllowDeath)?;

            let new_task = Task {
                ref_num: ref_num.clone(),
                delegate_tea_id,
                model_cid,
                body_cid,
                payment: neg_imbalance.peek(),
            };

            Tasks::<T>::insert(&ref_num, &new_task);

            Self::deposit_event(RawEvent::NewTaskAdded(sender, new_task));
		}

        #[weight = 0]
		pub fn complete_task(
		    origin,
		    ref_num: H256,
		    winner_tea_id: TeaPubKey,
		    delegate_sig: Vec<u8>,
		    result: Vec<u8>,
		    result_sig: Vec<u8>
		    ) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    // check if (sender, ephemeral_id) exist

            // check if the task exist
		    ensure!(Tasks::<T>::contains_key(&ref_num), Error::<T>::TaskNotExist);
		    let task = Tasks::<T>::get(&ref_num).unwrap();

            // check if the task status is in precessing

            // check the delegate signature
            Self::verify_delegate_sig(task.delegate_tea_id, delegate_sig, winner_tea_id, ref_num)?;

		    // check result signature
		    Self::verify_result_sig(winner_tea_id, result_sig, &result)?;

            let _positive_imbalance = T::Currency::deposit_creating(&sender, task.payment.clone());

            // task done

            Self::deposit_event(RawEvent::CompleteTask(sender, ref_num, result));

		    Ok(())
		}

        #[weight = 0]
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

        #[weight = 0]
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

		#[weight = 0]
		pub fn deposit(
		    origin,
            delegator_ephemeral_id: TeaPubKey,
            deposit_pub_key: TeaPubKey,
            delegator_signature: Vec<u8>,
            amount: BalanceOf<T>,
            expire_time: u64,
		) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

            debug::info!("sender: {:?}", sender);
            debug::info!("sender: {:#?}", sender);

		    // let public = sr25519::Public(*(sender as AccountId32).as_ref());
		    // let public = sr25519::Public::from_str(sender);
            //
            // ensure!(delegator_signature.len() == 64, Error::<T>::InvalidExecutorSig);
            // let signature = ed25519::Signature::from_slice(&delegator_signature[..]);

            // assert!(signature.verify(msg, &pair.public()));
            // signature.verify(&delegator_ephemeral_id[..], sender);
            // Verify::verify(&signature, &delegator_ephemeral_id[..], &public);

            // ensure!(sp_io::crypto::ed25519_verify(&signature, &delegator_ephemeral_id[..], &public),
            //         Error::<T>::InvalidExecutorSig);



		    // ensure delegator_ephemeral_id exist
		    ensure!(!DepositMap::<T>::contains_key((&sender, &deposit_pub_key)), Error::<T>::DepositAlreadyExist);

            let _neg_imbalance = T::Currency::withdraw(&sender,
		        amount,
		        WithdrawReasons::except(WithdrawReason::TransactionPayment),
		        ExistenceRequirement::AllowDeath)?;

            // if DepositMap::<T>::contains_key((&sender, &delegator_ephemeral_id)) {
            //     let mut deposit = DepositMap::<T>::get((&sender, &delegator_ephemeral_id)).unwrap();
		    //     ensure!(expire_time > deposit.expire_time + 100, Error::<T>::InvalidExpairTime);
		    //     ensure!(deposit_pub_key == deposit.deposit_pub_key, Error::<T>::InvalidDepositPubkey);
            //     deposit.amount += amount;
            //     deposit.expire_time = expire_time;
            //     DepositMap::<T>::insert((sender, delegator_ephemeral_id), deposit);
            // } else {
                // valid if expire GT current block number

                // let new_deposit = Deposit {
                //     delegator_ephemeral_id,
                //     deposit_pub_key,
                //     delegator_signature,
                //     amount,
                //     expire_time,
                // };
                // DepositMap::<T>::insert((&sender, &deposit_pub_key), &new_deposit);
            // }

            // Self::deposit_event(RawEvent::NewDepositAdded(sender, new_deposit));

            Ok(())
		}

		#[weight = 0]
		pub fn settle_accounts(
		    origin,
		    // use Lookup
            employer: T::AccountId,
            delegator_ephemeral_id: TeaPubKey,
            errand_uuid: Vec<u8>,
            payment: BalanceOf<T>,
            payment_type: u32,
            employer_sig: Vec<u8>,
            executor_ephemeral_id: TeaPubKey,
            expired_time: u64,
            delegate_signature: Vec<u8>,
            result_cid: Cid,
            executor_singature: Vec<u8>,
		) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    ensure!(DepositMap::<T>::contains_key((&employer, &delegator_ephemeral_id)), Error::<T>::DelegatorNotExist);

            let mut deposit = DepositMap::<T>::get((&employer, &delegator_ephemeral_id)).unwrap();
            ensure!(deposit.amount > payment, Error::<T>::InsufficientDeposit);
            deposit.amount -= payment;
            DepositMap::<T>::insert((&employer, &delegator_ephemeral_id), deposit);

            debug::info!("deposit_creating payment: {:?}", payment);
            let _positive_imbalance = T::Currency::deposit_creating(&sender, payment);

            let bill = Bill {
                employer,
                delegator_ephemeral_id,
                errand_uuid,
                payment,
                payment_type,
                executor_ephemeral_id,
                expired_time,
                result_cid,
            };
            Self::deposit_event(RawEvent::SettleAccounts(sender, bill));

            Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
    fn get_task_id(sender: &T::AccountId, task: &Task<BalanceOf<T>>) -> H256 {
        let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
        let payload = (
            random_seed,
            sender.clone(),
            task,
            <system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_256).into()
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
