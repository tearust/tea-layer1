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
                    StorageMap, StorageValue, ensure,
                    traits::{Randomness, Currency, ExistenceRequirement, WithdrawReason, WithdrawReasons,
                             OnUnbalanced, Imbalance}};
use sp_std::prelude::*;
use sp_io::hashing::blake2_256;
use sp_core::{crypto, ed25519, sr25519, ecdsa, hash::{H256, H512}};
use pallet_balances as balances;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
// type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type TeaId = Vec<u8>;
type PeerId = Vec<u8>;

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
pub struct Task<Balance> {
    delegate_node: TeaId,
    model_cid: Vec<u8>,
    body_cid: Vec<u8>,
    payment: Balance,
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
			map hasher(blake2_256) H256 => Option<Task<BalanceOf<T>>>;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
	{
		NewNodeJoined(AccountId, Node),
		UpdateNodePeer(AccountId, Node),
		NewModelAdded(AccountId),
		NewTaskAdded(AccountId, Node, H256, Task<Balance>),
		CompleteTask(AccountId, Task<Balance>, Balance),
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

		pub fn add_new_task(origin,
		    delegate_node: TeaId,
		    model_cid: Vec<u8>,
		    body_cid: Vec<u8>,
		    payment: BalanceOf<T>)
		{
			let sender = ensure_signed(origin)?;

		    ensure!(Nodes::contains_key(&delegate_node), Error::<T>::NodeNotExist);
		    let node = Nodes::get(&delegate_node).unwrap();

            let neg_imbalance = T::Currency::withdraw(&sender,
		        payment,
		        WithdrawReasons::except(WithdrawReason::TransactionPayment),
		        ExistenceRequirement::AllowDeath)?;

            let new_task = Task {
                delegate_node,
                model_cid,
                body_cid,
                payment: neg_imbalance.peek(),
            };
            let task_id = Self::get_task_id(&sender, &new_task);

            Tasks::<T>::insert(task_id, &new_task);

            Self::deposit_event(RawEvent::NewTaskAdded(sender, node, task_id, new_task));
		}

		pub fn complete_task(origin, task_id: H256) -> dispatch::DispatchResult {
		    let sender = ensure_signed(origin)?;

		    ensure!(Tasks::<T>::contains_key(&task_id), Error::<T>::TaskNotExist);
		    let task = Tasks::<T>::get(&task_id).unwrap();

            let positive_imbalance = T::Currency::deposit_creating(&sender, task.payment.clone());

            Self::deposit_event(RawEvent::CompleteTask(sender, task, positive_imbalance.peek()));

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
}
