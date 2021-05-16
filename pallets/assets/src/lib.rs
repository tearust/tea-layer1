#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module, decl_event, decl_storage, decl_error, ensure,
	Parameter, RuntimeDebug, weights::GetDispatchInfo,
	traits::{Currency, ReservableCurrency, Get, BalanceStatus},
	dispatch::PostDispatchInfo,
	debug,
};
use sp_std::prelude::*;
use sp_runtime::traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, StaticLookup};
use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::traits::One;
use codec::{Encode, Decode};

#[cfg(test)]
mod tests;

/// The module configuration trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The units in which we record balances.
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	/// The arithmetic type of asset identifier.
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		/// Issue a new class of fungible assets. There are, and will only ever be, `total`
		/// such assets and they'll all belong to the `origin` initially. It will have an
		/// identifier `AssetId` instance: this will be specified in the `Issued` event.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 storage mutation (codec `O(1)`).
		/// - 2 storage writes (condec `O(1)`).
		/// - 1 event.
		/// # </weight>
		// #[weight = 0]
		// fn issue(origin, #[compact] total: T::Balance) {
		// 	let origin = ensure_signed(origin)?;

		// 	let id = Self::next_asset_id();
		// 	<NextAssetId<T>>::mutate(|id| *id += One::one());

		// 	<Balances<T>>::insert((id, &origin), total);
		// 	<TotalSupply<T>>::insert(id, total);

		// 	Self::deposit_event(RawEvent::Issued(id, origin, total));
		// }

		/// Move some assets from one holder to another.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 static lookup
		/// - 2 storage mutations (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		// #[weight = 0]
		// fn transfer(origin,
		// 	#[compact] id: T::AssetId,
		// 	target: <T::Lookup as StaticLookup>::Source,
		// 	#[compact] amount: T::Balance
		// ) {
		// 	let origin = ensure_signed(origin)?;
		// 	let origin_account = (id, origin.clone());
		// 	let origin_balance = <Balances<T>>::get(&origin_account);
		// 	let target = T::Lookup::lookup(target)?;
		// 	ensure!(!amount.is_zero(), Error::<T>::AmountZero);
		// 	ensure!(origin_balance >= amount, Error::<T>::BalanceLow);

		// 	Self::deposit_event(RawEvent::Transferred(id, origin, target.clone(), amount));
		// 	<Balances<T>>::insert(origin_account, origin_balance - amount);
		// 	<Balances<T>>::mutate((id, target), |balance| *balance += amount);
		// }

		/// Destroy any assets of `id` owned by `origin`.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 storage mutation (codec `O(1)`).
		/// - 1 storage deletion (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		// #[weight = 0]
		// fn destroy(origin, #[compact] id: T::AssetId) {
		// 	let origin = ensure_signed(origin)?;
		// 	let balance = <Balances<T>>::take((id, &origin));
		// 	ensure!(!balance.is_zero(), Error::<T>::BalanceZero);

		// 	<TotalSupply<T>>::mutate(id, |total_supply| *total_supply -= balance);
		// 	Self::deposit_event(RawEvent::Destroyed(id, origin, balance));
    // }
    
    #[weight = 0]
    fn test_add_pcml(sender) {
      let sender = ensure_signed(sender)?;

      let pcml = Self::new_pcml();
      debug::info!("===> {:?}", pcml);

      Self::add_pcml_to_account(sender, pcml);
    }
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AssetId,
	{
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(AssetId, AccountId, Balance),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		Transferred(AssetId, AccountId, AccountId, Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Destroyed(AssetId, AccountId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Transfer amount should be non-zero
		AmountZero,
		/// Account balance must be greater than or equal to the transfer amount
		BalanceLow,
		/// Balance should be non-zero
		BalanceZero,
	}
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug)]
pub struct PCML<AssetId, BlockNumber> {
  id: AssetId,
  group: Vec<u8>,
  created_at: BlockNumber,
  // updated_at: BlockNumber,
}

decl_storage! {
	trait Store for Module<T: Trait> as Assets {
		/// The number of units of assets held by any given account.
    Balances: 
      map 
        hasher(blake2_128_concat) (T::AssetId, T::AccountId) 
      => 
        T::Balance;
		/// The next asset identifier up for grabs.
		LastAssetId: T::AssetId;
		/// The total unit supply of an asset.
		///
		/// TWOX-NOTE: `AssetId` is trusted, so this is safe.
    TotalSupply: map hasher(twox_64_concat) T::AssetId => T::Balance;
    
    PCMLAll: 
      map 
        hasher(twox_64_concat) T::AccountId
      => 
        Vec<PCML<T::AssetId, T::BlockNumber>>;
  }
  
  add_extra_genesis {
    config(pcml_list): Vec<(T::AccountId, u32)>;
    build(|config: &Self| {
      for (account, n) in config.pcml_list.iter() {
        let pcml = Module::<T>::new_pcml();
        Module::<T>::add_pcml_to_account(account.to_owned(), pcml);
      }
    })
  }
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
	// Public immutables

	/// Get the asset `id` balance of `who`.
	pub fn balance(id: T::AssetId, who: T::AccountId) -> T::Balance {
		<Balances<T>>::get((id, who))
	}

	/// Get the total supply of an asset `id`.
	pub fn total_supply(id: T::AssetId) -> T::Balance {
		<TotalSupply<T>>::get(id)
  }
  
  pub fn get_next_id() -> T::AssetId {
    let cid = <LastAssetId<T>>::get();
    let id = cid.clone();
    <LastAssetId<T>>::mutate(|id| *id += One::one());

    cid
  }

  pub fn new_pcml() -> PCML<T::AssetId, T::BlockNumber> {
    let id = Self::get_next_id();

    PCML {
      id,
      group: b"nitro".to_vec(),
      created_at: <system::Module<T>>::block_number(),
    }
  }

  pub fn add_pcml_to_account(
    who: T::AccountId,
    pcml: PCML<T::AssetId, T::BlockNumber>
  ) {
    if PCMLAll::<T>::contains_key(&who) {
      let mut list = PCMLAll::<T>::take(&who);
      list.push(pcml);
      PCMLAll::<T>::insert(&who, list);
    } 
    else {
      PCMLAll::<T>::insert(&who, vec![pcml]);
    }
  }
}

