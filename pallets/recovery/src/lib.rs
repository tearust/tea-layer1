#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_runtime::{
	traits::{Dispatchable, SaturatedConversion, CheckedAdd, CheckedMul},
	DispatchResult
};
use codec::{Encode, Decode};

use frame_support::{
	decl_module, decl_event, decl_storage, decl_error, ensure,
	Parameter, RuntimeDebug, weights::GetDispatchInfo,
	traits::{Currency, ReservableCurrency, Get, BalanceStatus},
	dispatch::PostDispatchInfo,
	debug,
};
use frame_system::{self as system, ensure_signed, ensure_root};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configuration trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The overarching call type.
	type Call: Parameter + Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo> + GetDispatchInfo;

	/// The currency mechanism.
	type Currency: ReservableCurrency<Self::AccountId>;

	/// The base amount of currency needed to reserve for creating a recovery configuration.
	///
	/// This is held for an additional storage item whose value size is
	/// `2 + sizeof(BlockNumber, Balance)` bytes.
	type ConfigDepositBase: Get<BalanceOf<Self>>;

	/// The amount of currency needed per additional user when creating a recovery configuration.
	///
	/// This is held for adding `sizeof(AccountId)` bytes more into a pre-existing storage value.
	type FriendDepositFactor: Get<BalanceOf<Self>>;

	/// The maximum amount of friends allowed in a recovery configuration.
	type MaxFriends: Get<u16>;

	/// The base amount of currency needed to reserve for starting a recovery.
	///
	/// This is primarily held for deterring malicious recovery attempts, and should
	/// have a value large enough that a bad actor would choose not to place this
	/// deposit. It also acts to fund additional storage item whose value size is
	/// `sizeof(BlockNumber, Balance + T * AccountId)` bytes. Where T is a configurable
	/// threshold.
	type RecoveryDeposit: Get<BalanceOf<Self>>;

}

/// An active recovery process.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug)]
pub struct ActiveRecovery<BlockNumber, Balance, AccountId> {
	/// The block number when the recovery process started.
	created: BlockNumber,
	/// The amount held in reserve of the `depositor`,
	/// To be returned once this recovery process is closed.
	deposit: Balance,
	/// The friends which have vouched so far. Always sorted.
	friends: Vec<AccountId>,
}

/// Configuration for recovering an account.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug)]
pub struct RecoveryConfig<BlockNumber, Balance, AccountId> {
	/// The minimum number of blocks since the start of the recovery process before the account
	/// can be recovered.
	delay_period: BlockNumber,
	/// The amount held in reserve of the `depositor`,
	/// to be returned once this configuration is removed.
	deposit: Balance,
	/// The list of friends which can help recover an account. Always sorted.
	friends: Vec<AccountId>,
	/// The number of approving friends needed to recover an account.
	threshold: u16,
}

decl_storage! {
	trait Store for Module<T: Trait> as Recovery {
		/// The set of recoverable accounts and their recovery configuration.
		pub Recoverable get(fn recovery_config):
			map hasher(twox_64_concat) T::AccountId
			=> Option<RecoveryConfig<T::BlockNumber, BalanceOf<T>, T::AccountId>>;

		/// Active recovery attempts.
		///
		/// First account is the account to be recovered, and the second account
		/// is the user trying to recover the account.
		pub ActiveRecoveries get(fn active_recovery):
			double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) T::AccountId =>
			Option<ActiveRecovery<T::BlockNumber, BalanceOf<T>, T::AccountId>>;

		/// The list of allowed proxy accounts.
		///
		/// Map from the user who can access it to the recovered account.
		pub Proxy get(fn proxy):
			map hasher(blake2_128_concat) T::AccountId => Option<T::AccountId>;
	}
}

decl_event! {
	/// Events type.
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// A recovery process has been set up for an \[account\].
		RecoveryCreated(AccountId),
		/// A recovery process has been initiated for lost account by rescuer account.
		/// \[lost, rescuer\]
		RecoveryInitiated(AccountId, AccountId),
		/// A recovery process for lost account by rescuer account has been vouched for by sender.
		/// \[lost, rescuer, sender\]
		RecoveryVouched(AccountId, AccountId, AccountId),
		/// A recovery process for lost account by rescuer account has been closed.
		/// \[lost, rescuer\]
		RecoveryClosed(AccountId, AccountId),
		/// Lost account has been successfully recovered by rescuer account.
		/// \[lost, rescuer\]
		AccountRecovered(AccountId, AccountId),
		/// A recovery process has been removed for an \[account\].
		RecoveryRemoved(AccountId),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// User is not allowed to make a call on behalf of this account
		NotAllowed,
		/// Threshold must be greater than zero
		ZeroThreshold,
		/// Friends list must be greater than zero and threshold
		NotEnoughFriends,
		/// Friends list must be less than max friends
		MaxFriends,
		/// Friends list must be sorted and free of duplicates
		NotSorted,
		/// This account is not set up for recovery
		NotRecoverable,
		/// This account is already set up for recovery
		AlreadyRecoverable,
		/// A recovery process has already started for this account
		AlreadyStarted,
		/// A recovery process has not started for this rescuer
		NotStarted,
		/// This account is not a friend who can vouch
		NotFriend,
		/// The friend must wait until the delay period to vouch for this recovery
		DelayPeriod,
		/// This user has already vouched for this recovery
		AlreadyVouched,
		/// The threshold for recovering this account has not been met
		Threshold,
		/// There are still active recovery attempts that need to be closed
		StillActive,
		/// There was an overflow in a calculation
		Overflow,
		/// This account is already set up for recovery
		AlreadyProxy,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		/// The base amount of currency needed to reserve for creating a recovery configuration.
		const ConfigDepositBase: BalanceOf<T> = T::ConfigDepositBase::get();

		/// The amount of currency needed per additional user when creating a recovery configuration.
		const FriendDepositFactor: BalanceOf<T> = T::FriendDepositFactor::get();

		/// The maximum amount of friends allowed in a recovery configuration.
		// const MaxFriends: u16 = T::MaxFriends::get();

		/// The base amount of currency needed to reserve for starting a recovery.
		const RecoveryDeposit: BalanceOf<T> = T::RecoveryDeposit::get();

		/// Deposit one of this module's events by using the default implementation.
		fn deposit_event() = default;

		
		#[weight = (call.get_dispatch_info().weight + 10_000, call.get_dispatch_info().class)]
		fn as_recovered(origin,
			account: T::AccountId,
			call: Box<<T as Trait>::Call>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Check `who` is allowed to make a call on behalf of `account`
			let target = Self::proxy(&who).ok_or(Error::<T>::NotAllowed)?;
			ensure!(target == account, Error::<T>::NotAllowed);
			call.dispatch(frame_system::RawOrigin::Signed(account).into())
				.map(|_| ()).map_err(|e| e.error)
		}

		/// Allow ROOT to bypass the recovery process and set an a rescuer account
		/// for a lost account directly.
		///
		/// The dispatch origin for this call must be _ROOT_.
		///
		/// Parameters:
		/// - `lost`: The "lost account" to be recovered.
		/// - `rescuer`: The "rescuer account" which can call as the lost account.
		///
		/// # <weight>
		/// - One storage write O(1)
		/// - One event
		/// # </weight>
		#[weight = 0]
		fn set_recovered(origin, lost: T::AccountId, rescuer: T::AccountId) {
			ensure_root(origin)?;
			// Create the recovery storage item.
			<Proxy<T>>::insert(&rescuer, &lost);
			Self::deposit_event(RawEvent::AccountRecovered(lost, rescuer));
		}

		
		#[weight = 10_000]
		fn create_recovery(origin,
			friends: Vec<T::AccountId>,
			threshold: u16,
			delay_period: T::BlockNumber
		) {
			let who = ensure_signed(origin)?;
			debug::info!("Start create_recovery");

			// Check account is not already set up for recovery
			ensure!(!<Recoverable<T>>::contains_key(&who), Error::<T>::AlreadyRecoverable);
			// Check user input is valid
			ensure!(threshold >= 1, Error::<T>::ZeroThreshold);
			ensure!(!friends.is_empty(), Error::<T>::NotEnoughFriends);
			ensure!(threshold as usize <= friends.len(), Error::<T>::NotEnoughFriends);

			let max_friends = T::MaxFriends::get() as usize;
			ensure!(friends.len() <= max_friends, Error::<T>::MaxFriends);

			
			ensure!(Self::is_sorted_and_unique(&friends), Error::<T>::NotSorted);

			// Total deposit is base fee + number of friends * factor fee
			let friend_deposit = T::FriendDepositFactor::get()
				.checked_mul(&friends.len().saturated_into())
				.ok_or(Error::<T>::Overflow)?;
			let total_deposit = T::ConfigDepositBase::get()
				.checked_add(&friend_deposit)
				.ok_or(Error::<T>::Overflow)?;
			// Reserve the deposit
			T::Currency::reserve(&who, total_deposit)?;
			// Create the recovery configuration
			let recovery_config = RecoveryConfig {
				delay_period,
				deposit: total_deposit,
				friends,
				threshold,
			};
			// debug::info!("friend_deposit => {:?}", friend_deposit/T::TokenUnit::get());
			// debug::info!("total_deposit => {:?}", total_deposit/T::TokenUnit::get());
			// debug::info!("recovery_config => {:?}", recovery_config);
			// Create the recovery configuration storage item
			<Recoverable<T>>::insert(&who, recovery_config);

			Self::deposit_event(RawEvent::RecoveryCreated(who));
		}

		#[weight = 100_000]
		fn initiate_recovery(origin, account: T::AccountId) {
			let who = ensure_signed(origin)?;
			// Check that the account is recoverable
			ensure!(<Recoverable<T>>::contains_key(&account), Error::<T>::NotRecoverable);
			// Check that the recovery process has not already been started
			ensure!(!<ActiveRecoveries<T>>::contains_key(&account, &who), Error::<T>::AlreadyStarted);
			// Take recovery deposit
			let recovery_deposit = T::RecoveryDeposit::get();
			T::Currency::reserve(&who, recovery_deposit)?;
			// Create an active recovery status
			let recovery_status = ActiveRecovery {
				created: <system::Module<T>>::block_number(),
				deposit: recovery_deposit,
				friends: vec![],
			};

			// debug::info!("recovery_deposit => {:?}", recovery_deposit/T::TokenUnit::get());
			// debug::info!("recovery_status => {:?}", recovery_status);

			// Create the active recovery storage item
			<ActiveRecoveries<T>>::insert(&account, &who, recovery_status);
			Self::deposit_event(RawEvent::RecoveryInitiated(account, who));
		}

		
		#[weight = 100_000]
		fn vouch_recovery(origin, lost: T::AccountId, rescuer: T::AccountId) {
			let who = ensure_signed(origin)?;
			// Get the recovery configuration for the lost account.
			let recovery_config = Self::recovery_config(&lost).ok_or(Error::<T>::NotRecoverable)?;
			// Get the active recovery process for the rescuer.
			let mut active_recovery = Self::active_recovery(&lost, &rescuer).ok_or(Error::<T>::NotStarted)?;
			
			// Make sure the voter is a friend
			ensure!(Self::is_friend(&recovery_config.friends, &who), Error::<T>::NotFriend);

			// Either insert the vouch, or return an error that the user already vouched.
			match active_recovery.friends.binary_search(&who) {
				Ok(_pos) => return Err(Error::<T>::AlreadyVouched.into()),
				Err(pos) => active_recovery.friends.insert(pos, who.clone()),
			}
			// Update storage with the latest details
			<ActiveRecoveries<T>>::insert(&lost, &rescuer, active_recovery);
			Self::deposit_event(RawEvent::RecoveryVouched(lost, rescuer, who));
		}

		
		#[weight = 100_000]
		fn claim_recovery(origin, account: T::AccountId) {
			let who = ensure_signed(origin)?;
			// Get the recovery configuration for the lost account
			let recovery_config = Self::recovery_config(&account).ok_or(Error::<T>::NotRecoverable)?;
			// Get the active recovery process for the rescuer
			let active_recovery = Self::active_recovery(&account, &who).ok_or(Error::<T>::NotStarted)?;
			ensure!(!Proxy::<T>::contains_key(&who), Error::<T>::AlreadyProxy);
			// Make sure the delay period has passed
			let current_block_number = <system::Module<T>>::block_number();
			let recoverable_block_number = active_recovery.created
				.checked_add(&recovery_config.delay_period)
				.ok_or(Error::<T>::Overflow)?;
			ensure!(recoverable_block_number <= current_block_number, Error::<T>::DelayPeriod);
			// Make sure the threshold is met
			ensure!(
				recovery_config.threshold as usize <= active_recovery.friends.len(),
				Error::<T>::Threshold
			);
			// Create the recovery storage item
			Proxy::<T>::insert(&who, &account);
			system::Module::<T>::inc_ref(&who);
			Self::deposit_event(RawEvent::AccountRecovered(account, who));
		}

		/// As the controller of a recoverable account, close an active recovery
		/// process for your account.
		///
		/// Payment: By calling this function, the recoverable account will receive
		/// the recovery deposit `RecoveryDeposit` placed by the rescuer.
		///
		/// The dispatch origin for this call must be _Signed_ and must be a
		/// recoverable account with an active recovery process for it.
		///
		/// Parameters:
		/// - `rescuer`: The account trying to rescue this recoverable account.
		///
		/// # <weight>
		/// Key: V (len of vouching friends)
		/// - One storage read/remove to get the active recovery process. O(1), Codec O(V)
		/// - One balance call to repatriate reserved. O(X)
		/// - One event.
		///
		/// Total Complexity: O(V + X)
		/// # </weight>
		#[weight = 30_000]
		fn close_recovery(origin, rescuer: T::AccountId) {
			let who = ensure_signed(origin)?;
			// Take the active recovery process started by the rescuer for this account.
			let active_recovery = <ActiveRecoveries<T>>::take(&who, &rescuer).ok_or(Error::<T>::NotStarted)?;
			// Move the reserved funds from the rescuer to the rescued account.
			// Acts like a slashing mechanism for those who try to maliciously recover accounts.
			let _ = T::Currency::repatriate_reserved(&rescuer, &who, active_recovery.deposit, BalanceStatus::Free);
			Self::deposit_event(RawEvent::RecoveryClosed(who, rescuer));
		}

		/// Remove the recovery process for your account. Recovered accounts are still accessible.
		///
		/// NOTE: The user must make sure to call `close_recovery` on all active
		/// recovery attempts before calling this function else it will fail.
		///
		/// Payment: By calling this function the recoverable account will unreserve
		/// their recovery configuration deposit.
		/// (`ConfigDepositBase` + `FriendDepositFactor` * #_of_friends)
		///
		/// The dispatch origin for this call must be _Signed_ and must be a
		/// recoverable account (i.e. has a recovery configuration).
		///
		/// # <weight>
		/// Key: F (len of friends)
		/// - One storage read to get the prefix iterator for active recoveries. O(1)
		/// - One storage read/remove to get the recovery configuration. O(1), Codec O(F)
		/// - One balance call to unreserved. O(X)
		/// - One event.
		///
		/// Total Complexity: O(F + X)
		/// # </weight>
		#[weight = 30_000]
		fn remove_recovery(origin) {
			let who = ensure_signed(origin)?;
			// Check there are no active recoveries
			let mut active_recoveries = <ActiveRecoveries<T>>::iter_prefix_values(&who);
			ensure!(active_recoveries.next().is_none(), Error::<T>::StillActive);
			// Take the recovery configuration for this account.
			let recovery_config = <Recoverable<T>>::take(&who).ok_or(Error::<T>::NotRecoverable)?;

			// Unreserve the initial deposit for the recovery configuration.
			T::Currency::unreserve(&who, recovery_config.deposit);
			Self::deposit_event(RawEvent::RecoveryRemoved(who));
		}

		/// Cancel the ability to use `as_recovered` for `account`.
		///
		/// The dispatch origin for this call must be _Signed_ and registered to
		/// be able to make calls on behalf of the recovered account.
		///
		/// Parameters:
		/// - `account`: The recovered account you are able to call on-behalf-of.
		///
		/// # <weight>
		/// - One storage mutation to check account is recovered by `who`. O(1)
		/// # </weight>
		#[weight = 0]
		fn cancel_recovered(origin, account: T::AccountId) {
			let who = ensure_signed(origin)?;
			// Check `who` is allowed to make a call on behalf of `account`
			ensure!(Self::proxy(&who) == Some(account), Error::<T>::NotAllowed);
			Proxy::<T>::remove(&who);
			system::Module::<T>::dec_ref(&who);
		}
	}
}

impl<T: Trait> Module<T> {
	/// Check that friends list is sorted and has no duplicates.
	fn is_sorted_and_unique(friends: &[T::AccountId]) -> bool {
		friends.windows(2).all(|w| w[0] < w[1])
	}

	/// Check that a user is a friend in the friends list.
	fn is_friend(friends: &[T::AccountId], friend: &T::AccountId) -> bool {
		friends.binary_search(&friend).is_ok()
	}


}
