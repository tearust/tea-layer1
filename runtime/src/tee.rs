use system::ensure_signed;
use codec::{Decode, Encode};
use support::{decl_event, decl_module, decl_storage, dispatch::Result,
              ensure, StorageMap, StorageValue, traits::Currency};
use rstd::vec::Vec;

pub trait Trait: balances::Trait {
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
    value: Vec<u8>,
    task_id: Hash,
    block_height: u64,
    pub_key: Hash,
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		PendingNodes get(panding_nodes): map T::AccountId => Node<T::AccountId, T::Balance>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event() = default;

		pub fn new_node_join(origin, node_account: T::AccountId, deposit_amt: T::Balance) -> Result {
		    let _ = ensure_signed(origin)?;
            let new_node = Node {
                account: node_account.clone(),
                amt: deposit_amt,
            };
            <PendingNodes<T>>::insert(node_account.clone(), new_node);

            Self::deposit_event(RawEvent::NewNodeJoin(node_account, deposit_amt));
		    Ok(())
		}

		pub fn remote_attestation_done(origin) {

		}

		pub fn update_lambda(origin) {

		}

		pub fn compute_task(origin) {

		}

		pub fn compute_task_winner_app(origin) {

		}

		pub fn compute_task_execution_done(origin) {

		}

		pub fn compute_task_ra_done(origin) {

		}

		pub fn compute_task_owner_confirmation_done(origin) {

		}

		pub fn gas_transfer(origin, to: T::AccountId, gas: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;
            <balances::Module<T> as Currency<_>>::transfer(&sender, &to, gas)?;

			Self::deposit_event(RawEvent::GasTransfered(sender, to, gas));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T>
	where
	    AccountId = <T as system::Trait>::AccountId,
	    Balance = <T as balances::Trait>::Balance,
	{
		GasTransfered(AccountId, AccountId, Balance),
		NewNodeJoin(AccountId, Balance),
	}
);
