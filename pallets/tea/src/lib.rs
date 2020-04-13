#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use system::ensure_signed;
use codec::{Decode, Encode};
use frame_support::{decl_event, decl_module, decl_storage, decl_error, dispatch,
			  ensure, StorageMap, StorageValue, traits::{Randomness, Currency}};
use sp_std::vec::Vec;
use sp_io::hashing::blake2_256;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use pallet_balances as balances;

/// The pallet's configuration trait.
pub trait Trait: balances::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Node<AccountId, Balance> {
	account: AccountId,
	amt: Balance,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ProofOfVrf<Hash> {
	j: u64,
	proof: Vec<u8>,
	pub_key: Hash,
	value: Vec<u8>,
	task_id: Hash,
	block_height: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Model<AccountId> {
	account: AccountId,
	payment: u32,
	cid: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Task<AccountId, Balance> {
	account: AccountId,
	amt: Balance,
	model_id: Vec<u8>,
	cid: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Competitor<AccountId> {
	account: AccountId,
	task_id: Vec<u8>,
    random_value: [u8; 32],
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		Nodes get(nodes):
			map hasher(blake2_256) T::AccountId => Node<T::AccountId, T::Balance>;
		Models get(models):
			map hasher(blake2_256) Vec<u8> => Model<T::AccountId>;
		Tasks get(tasks):
			map hasher(blake2_256) Vec<u8> => Task<T::AccountId, T::Balance>;
		TaskCompetitions get(task_competitions):
			map hasher(blake2_256) Vec<u8> => Vec<Competitor<T::AccountId>>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	  	Balance = <T as balances::Trait>::Balance,
	{
		NewNodeJoined(AccountId, Balance),
		NewLambadaAdded(AccountId),
		NewTaskAdded(AccountId),
		NewCompetitorAdded(AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
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

		pub fn new_node_join(origin, deposit_amt: T::Balance) {
		    let sender = ensure_signed(origin)?;
            let new_node = Node {
                account: sender.clone(),
                amt: deposit_amt,
            };
            <Nodes<T>>::insert(sender.clone(), new_node);
            Self::deposit_event(RawEvent::NewNodeJoined(sender, deposit_amt));
		}

		pub fn remote_attestation_done(origin) {
		    let _ = ensure_signed(origin)?;
		}

		pub fn update_lambda(origin, payment: u32, cid: Vec<u8>) {
		    let sender = ensure_signed(origin)?;
            let new_model = Model {
                account: sender.clone(),
                payment,
                cid: cid.clone(),
            };
            <Models<T>>::insert(cid, new_model);
            Self::deposit_event(RawEvent::NewLambadaAdded(sender));
		}

		pub fn compute_task( origin, amt: T::Balance, model_id: Vec<u8>, cid: Vec<u8>) {
		    let sender = ensure_signed(origin)?;
            let new_task = Task {
                account: sender.clone(),
                amt,
                model_id,
                cid: cid.clone(),
            };
            <Tasks<T>>::insert(cid, new_task);
            Self::deposit_event(RawEvent::NewTaskAdded(sender));
		}

		pub fn compute_task_winner_app(origin, task_id: Vec<u8>) -> dispatch::DispatchResult  {
		    let sender = ensure_signed(origin)?;
		    let random_value = Self::random_value(&sender, task_id.clone());
            let new_competitor = Competitor {
                account: sender.clone(),
                task_id: task_id.clone(),
                random_value,
            };
            let mut competitors = Self::task_competitions(task_id.clone());
            competitors.push(new_competitor);
            <TaskCompetitions<T>>::insert(task_id, competitors);
            Self::deposit_event(RawEvent::NewCompetitorAdded(sender));
			Ok(())
		}

		pub fn compute_task_execution_done(origin) {
		    let _ = ensure_signed(origin)?;

		}

		pub fn compute_task_ra_done(origin) {
		    let _ = ensure_signed(origin)?;

		}

		pub fn compute_task_owner_confirmation_done(origin) {
		    let _ = ensure_signed(origin)?;

		}
	}
}

impl<T: Trait> Module<T> {
	fn random_value(sender: &T::AccountId, task_id: Vec<u8>) -> [u8; 32] {
		let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
		let payload = (
            random_seed,
			sender.clone(),
			task_id,
			<system::Module<T>>::block_number(),
		);
		payload.using_encoded(blake2_256)
	}

	pub fn get_sum() -> u32 {
        100 + 200
	}
}

sp_api::decl_runtime_apis! {
    pub trait TeaApi {
        fn get_sum() -> u32;
    }
}