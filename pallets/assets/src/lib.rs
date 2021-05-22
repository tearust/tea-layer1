#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module, decl_event, decl_storage, decl_error, ensure, 
	Parameter, RuntimeDebug, weights::GetDispatchInfo,
	traits::{Currency, LockableCurrency, BalanceStatus, Get,},
	dispatch,
	debug,
};

use sp_std::prelude::*;
use sp_runtime::traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, StaticLookup};
use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::traits::One;
use codec::{Encode, Decode};

use pallet_balances as balances;

#[cfg(test)]
mod tests;

// pub const DOLLARS: u128 = 1_000_000_000_000;
// pub const StakingPrice: u32 = 1000;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// The module configuration trait.
pub trait Trait: frame_system::Trait + balances::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;


	/// The arithmetic type of asset identifier.
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;
	
	// type LockableCurrency: LockableCurrency<Self::AccountId>;

	type Currency: Currency<Self::AccountId>;
  
  // Id coin for pre-sale, convert to CML when main-net onboard.
	type Dai: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	type Unit: Get<BalanceOf<Self>>;
	type StakingPrice: Get<u32>;
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
    
    // #[weight = 0]
    // fn test_add_pcml(sender) {
    //   let sender = ensure_signed(sender)?;

    //   let pcml = Self::new_pcml();
    //   debug::info!("===> {:?}", pcml);

    //   Self::add_pcml_to_account(sender, pcml);
    // }

    #[weight = 0]
    fn transfer_dai(
      sender, 
      target: T::AccountId,
      #[compact] amount: T::Dai,
    ) {
      let sender = ensure_signed(sender)?;

      let _sender_dai = Self::get_dai(&sender);
      let _target_dai = Self::get_dai(&target);

      ensure!(_sender_dai >= amount, Error::<T>::NotEnoughDai);

      Self::set_dai(&sender, _sender_dai-amount);
      Self::set_dai(&target, _target_dai+amount);
		}
		
		#[weight = 1_000]
		fn convert_cml_from_dai(
			sender,
		) {
			let sender = ensure_signed(sender)?;

			// check sender dai
			let _sender_dai = Self::get_dai(&sender);
			ensure!(_sender_dai > 0.into(), Error::<T>::NotEnoughDai);

			// TODO, check dai is frozen or live
			let status = b"Seed_Frozen".to_vec();

			// dai - 1
			Self::set_dai(&sender, _sender_dai-1.into());

			// add cml
			let cml = Self::new_cml_from_dai(b"nitro".to_vec(), status);
			Self::add_cml(&sender, cml);

		}

		#[weight = 1_000]
		fn active_cml_for_nitro(
			sender,
			cml_id: T::AssetId,
			miner_id: Vec<u8>,
			miner_ip: Vec<u8>,
		) -> dispatch::DispatchResult {
			let sender = ensure_signed(sender)?;

			let miner_item = MinerItem {
				id: miner_id.clone(),
				group: b"nitro".to_vec(),
				ip: miner_ip,
				status: b"active".to_vec(),
			};

			ensure!(!<MinerItemStore>::contains_key(&miner_id), Error::<T>::MinerAlreadyExist);

			let balance = T::Currency::free_balance(&sender);

			let max_price: BalanceOf<T> = T::Unit::get() * T::StakingPrice::get().into();
			ensure!(balance >= max_price, Error::<T>::NotEnoughTeaToStaking);


			let staking_item = StakingItem {
				owner: sender.clone(),
				category: b"tea".to_vec(),
				amount: T::StakingPrice::get(),
				cml: Vec::new(),
			};
			Self::updateCmlToActive(&sender, &cml_id, miner_id.clone(), staking_item)?;
			<MinerItemStore>::insert(&miner_id, miner_item);

		
			debug::info!("TODO ---- lock balance");

			Ok(())
		}
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		// <T as Trait>::Balance,
		<T as Trait>::AssetId,
	{
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(AssetId, AccountId),
		// /// Some assets were transferred. \[asset_id, from, to, amount\]
		// Transferred(AssetId, AccountId, AccountId, Balance),
		// /// Some assets were destroyed. \[asset_id, owner, balance\]
		// Destroyed(AssetId, AccountId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		NotEnoughDai,
		NotFoundCML,
		CMLNotLive,
		NotEnoughTeaToStaking,
		MinerAlreadyExist,
	}
}

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StakingItem<AccountId, AssetId> {
	owner: AccountId,
	category: Vec<u8>,   // seed, tea
	amount: u32,  // amount of tea
	cml: Vec<AssetId>,
}

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct MinerItem {
	id: Vec<u8>,
	group: Vec<u8>,
	ip: Vec<u8>,
	status: Vec<u8>,
}

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct CML<AssetId, AccountId, BlockNumber> {
  id: AssetId,
  group: Vec<u8>,   // nitro
	status: Vec<u8>,  // Seed_Live, Seed_Frozen, Seed_Planting, CML_Live
	life_time: BlockNumber, // whole life time for CML
	lock_time: BlockNumber, 
	mining_rate: u8, // 8 - 12, default 10
	staking_slot: Vec<StakingItem<AccountId, AssetId>>,
	created_at: BlockNumber,
	miner_id: Vec<u8>,
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
		LastAssetId: T::AssetId = 10000.into();
		/// The total unit supply of an asset.
		///
		/// TWOX-NOTE: `AssetId` is trusted, so this is safe.
    TotalSupply: map hasher(twox_64_concat) T::AssetId => T::Balance;
    
    CmlStore: 
      map 
        hasher(twox_64_concat) T::AccountId
      => 
        Vec<CML<T::AssetId, T::AccountId, T::BlockNumber>>;

    DaiStore:
      map
        hasher(twox_64_concat) T::AccountId
      =>
				T::Dai;
				
		MinerItemStore:
			map
				hasher(identity) Vec<u8>
			=>
				MinerItem;

  }
  
  add_extra_genesis {
    config(dai_list): Vec<(T::AccountId, T::Dai)>;
    build(|config: &Self| {
      for (account, amount) in config.dai_list.iter() {
        // let pcml = Module::<T>::new_pcml();
        // Module::<T>::add_pcml_to_account(account.to_owned(), pcml);
        Module::<T>::set_dai(&account, *amount);
      }
    })
  }
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
	// Public immutables

	/// Get the asset `id` balance of `who`.
	// pub fn balance(id: T::AssetId, who: T::AccountId) -> T::Balance {
	// 	<Balances<T>>::get((id, who))
	// }

	// /// Get the total supply of an asset `id`.
	// pub fn total_supply(id: T::AssetId) -> T::Balance {
	// 	<TotalSupply<T>>::get(id)
  // }
  
  pub fn get_next_id() -> T::AssetId {
    let cid = <LastAssetId<T>>::get();
    let id = cid.clone();
    <LastAssetId<T>>::mutate(|id| *id += One::one());

    cid
	}
	
	fn get_random_life() -> T::BlockNumber {
		10_000_000.into()
	}

	fn get_random_mining_rate() -> u8 {
		10 as u8
	}


  fn new_cml_from_dai(
		group: Vec<u8>,
		status: Vec<u8>,  // Seed_Live, Seed_Frozen
	) -> CML<T::AssetId, T::AccountId, T::BlockNumber> {

		// life time, lock time
		let current_block = <system::Module<T>>::block_number();
		let life_time = current_block + Self::get_random_life();
		let lock_time = 0.into();
		
		CML {
			id: Self::get_next_id(),
			group,
			status,
			mining_rate: Self::get_random_mining_rate(),
			life_time,
			lock_time,
			staking_slot: vec![],
			created_at: current_block,
			miner_id: b"".to_vec(),
		}

  }

  fn get_dai(who: &T::AccountId) -> T::Dai {
    let n = <DaiStore<T>>::get(&who);

    n
  }

  fn set_dai(
    who: &T::AccountId,
    amount: T::Dai
  ) {
    <DaiStore<T>>::mutate(&who, |n| *n = amount);
	}
	
	fn add_cml(
		who: &T::AccountId,
		cml: CML<T::AssetId, T::AccountId, T::BlockNumber>,
	) {
		if CmlStore::<T>::contains_key(&who) {
      let mut list = CmlStore::<T>::take(&who);
      list.insert(0, cml);
      CmlStore::<T>::insert(&who, list);
    } 
    else {
      CmlStore::<T>::insert(&who, vec![cml]);
    }
	}

	fn remove_cml_by_id() {}

	fn get_cml_list_by_account(
		who: &T::AccountId,
	) -> Vec<CML<T::AssetId, T::AccountId, T::BlockNumber>> {
		let list = {
			if <CmlStore<T>>::contains_key(&who) {
				CmlStore::<T>::get(&who)
			}
			else {
				vec![]
			}
		};
		
		list
	}

	fn set_cml_by_index(
		who: &T::AccountId,
		cml: CML<T::AssetId, T::AccountId, T::BlockNumber>,
		index: usize,
	) {
		CmlStore::<T>::mutate(&who, |list| {
			list.remove(index);
			list.insert(index, cml);

		});

		// let mut cml_list = CmlStore::<T>::take(&who);
		// cml_list.remove(index);
		// cml_list.insert(index, cml);

		// CmlStore::<T>::insert(&who, cml_list);
	}

	fn find_cml_index(
		who: &T::AccountId,
		cml_id: &T::AssetId,
	) -> (Vec<CML<T::AssetId, T::AccountId, T::BlockNumber>>, i32) {
		let list = Self::get_cml_list_by_account(&who);


		let index = match list.iter().position(|cml| cml.id == *cml_id) {
			Some(i) => i as i32,
			None => -1,
		};

		(list, index)
	}

	fn updateCmlToActive(
		who: &T::AccountId,
		cml_id: &T::AssetId,
		miner_id: Vec<u8>,
		staking_item: StakingItem<T::AccountId, T::AssetId>,
	) -> Result<(), Error<T>> {
		let (mut list, index) = Self::find_cml_index(&who, &cml_id);

		if(index < 0){
			return Err(Error::<T>::NotFoundCML);
		}

		let cml: &mut CML<T::AssetId, T::AccountId, T::BlockNumber> = list.get_mut(index as usize).unwrap();

		cml.status = b"CML_Live".to_vec();
		cml.miner_id = miner_id;

		cml.staking_slot.push(staking_item);

		Self::set_cml_by_index(&who, cml.clone(), index as usize);

		Ok(())
	}

	fn stakingToCml(
		staking_item: StakingItem<T::AccountId, T::AssetId>,

		who: &T::AccountId,
		target_cml_id: &T::AssetId,
	) -> Result<(), Error<T>> {
		let (mut list, index) = Self::find_cml_index(&who, &target_cml_id);

		if(index < 0){
			return Err(Error::<T>::NotFoundCML);
		}

		let cml: &mut CML<T::AssetId, T::AccountId, T::BlockNumber> = list.get_mut(index as usize).unwrap();

		if(cml.status != b"CML_Live".to_vec()){
			return Err(Error::<T>::CMLNotLive);
		}

		cml.staking_slot.push(staking_item);

		Self::set_cml_by_index(&who, cml.clone(), index as usize);
		
		Ok(())
	}


}

