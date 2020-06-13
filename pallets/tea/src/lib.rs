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
                    StorageMap, StorageValue, traits::{Randomness, Currency}, ensure};
use sp_std::prelude::*;
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

type TeaId = Vec<u8>;
type PeerId = Vec<u8>;
type TaskIndex = u32;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Node {
    tea_id: TeaId,
    peers: Vec<PeerId>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Model<AccountId> {
    account: AccountId,
    price: u32,
    cid: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Task {
    delegate_node: TeaId,
    ref_num: u32,
    cap_cid: Vec<u8>,
    model_cid: Vec<u8>,
    data_cid: Vec<u8>,
    payment: u32,
}

decl_storage! {
	trait Store for Module<T: Trait> as TeaModule {
		BootNodes get(bootnodes):
			Vec<Vec<u8>> = vec!["tea-node1".into(), "tea-node2".into()];

		Nodes get(nodes):
			map hasher(blake2_256) TeaId => Option<Node>;
		Models get(models):
			map hasher(blake2_256) Vec<u8> => Model<T::AccountId>;
		Tasks get(tasks):
			map hasher(blake2_256) TaskIndex => Option<Task>;
		TasksCount get(tasks_count): TaskIndex;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		NewNodeJoined(AccountId, Node),
		UpdateNodePeer(AccountId, Node),
		NewModelAdded(AccountId),
		NewTaskAdded(AccountId, Task),
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

		pub fn add_new_node(origin, tea_id: TeaId) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    ensure!(!Nodes::contains_key(&tea_id), Error::<T>::NodeAlreadyExist);

            let new_node = Node {
            	tea_id: tea_id.clone(),
            	peers: Vec::new(),
            };
            <Nodes>::insert(tea_id, &new_node);
            Self::deposit_event(RawEvent::NewNodeJoined(sender, new_node));

            Ok(())
		}

		pub fn update_peer_id(origin, tea_id: TeaId, peers: Vec<PeerId>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

		    ensure!(Nodes::contains_key(&tea_id), Error::<T>::NodeNotExist);

			let mut node = Nodes::get(&tea_id).unwrap();
        	node.peers = peers;
	        <Nodes>::insert(tea_id, &node);

            Self::deposit_event(RawEvent::UpdateNodePeer(sender, node));

            Ok(())
		}

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

		pub fn add_new_task(origin, delegate_node: TeaId, ref_num: u32,
		    cap_cid: Vec<u8>, model_cid: Vec<u8>, data_cid: Vec<u8>, payment: u32) {
			let sender = ensure_signed(origin)?;

			let next_task_index = Self::next_task_index()?;
            let new_task = Task {
                delegate_node,
                ref_num,
                cap_cid,
                model_cid,
                data_cid,
                payment,
            };

            Tasks::insert(next_task_index, &new_task);
            // fixme: maybe should use checked_add
            TasksCount::put(next_task_index + 1);
            Self::deposit_event(RawEvent::NewTaskAdded(sender, new_task));
		}

		pub fn complete_task(origin, task_id: Vec<u8>, proof: Vec<u8>) {
			let sender = ensure_signed(origin)?;

			// emit event with task id (finish)
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

    fn next_task_index() -> sp_std::result::Result<TaskIndex, dispatch::DispatchError> {
        let task_index = Self::tasks_count();
        ensure!(task_index != TaskIndex::MAX, Error::<T>::TaskCountOverflow);

        Ok(task_index)
    }
}
