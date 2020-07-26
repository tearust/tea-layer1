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
use frame_support::{decl_event, decl_module, decl_storage, decl_error, dispatch,
                    StorageMap, StorageValue, ensure, debug,
                    traits::{Randomness, Currency, ExistenceRequirement, WithdrawReason, WithdrawReasons,
                             Imbalance}};
use sp_std::prelude::*;
use sp_io::hashing::blake2_256;
use sp_core::{crypto, ed25519, hash::{H256}};
use pallet_balances as balances;

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

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct Node {
    tea_id: TeaPubKey,
    ephemeral_id: TeaPubKey,
    profile_cid: Vec<u8>,
    urls: Vec<Url>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Model<AccountId> {
    account: AccountId,
    price: u32,
    cid: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Task<Balance> {
    ref_num: H256,
    delegate_tea_id: TeaPubKey,
    model_cid: Vec<u8>,
    body_cid: Vec<u8>,
    payment: Balance,
}

decl_storage! {
	trait Store for Module<T: Trait> as TeaModule {
		Nodes get(fn nodes):
			map hasher(twox_64_concat) TeaPubKey => Option<Node>;

		BootstrapNodes get(fn bootstrap_nodes):
			map hasher(twox_64_concat) TeaPubKey => Option<Node>;

		EphemeralIds get(fn ephemera_ids):
		    map hasher(twox_64_concat) TeaPubKey => Option<TeaPubKey>;

		Models get(fn models):
			map hasher(blake2_128_concat) Vec<u8> => Model<T::AccountId>;
		Tasks get(fn tasks):
			map hasher(blake2_128_concat) H256 => Option<Task<BalanceOf<T>>>;
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
		    tea_sig: Vec<u8>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

		    ensure!(Nodes::contains_key(&tea_id) || BootstrapNodes::contains_key(&tea_id),
		        Error::<T>::NodeNotExist);
		    Self::verify_tea_sig(tea_id.clone(), tea_sig, ephemeral_id)?;

		    let urls_count = urls.len();
            let node = Node {
                tea_id: tea_id.clone(),
            	ephemeral_id,
            	profile_cid,
            	urls,
            };
            if urls_count == 0 {
                <Nodes>::insert(&tea_id, &node);
                <BootstrapNodes>::remove(&tea_id);
            } else {
	            <BootstrapNodes>::insert(&tea_id, &node);
                <Nodes>::remove(&tea_id);
            }

		    EphemeralIds::remove(&node.ephemeral_id);
	        EphemeralIds::insert(ephemeral_id, tea_id);

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
                let node = Self::nodes(id);
                return match node {
                    Some(n) => Some(n),
                    None => Self::bootstrap_nodes(id)
                }
            },
            None => None
        }
    }
}
