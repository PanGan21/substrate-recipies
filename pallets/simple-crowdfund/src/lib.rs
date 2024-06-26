#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{storage::child, traits::Currency, PalletId};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::Hasher;
use sp_runtime::traits::AccountIdConversion;

pub type FundIndex = u32;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type FundInfoOf<T> = FundInfo<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: PalletId = PalletId(*b"ex/cfund");

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::{AccountIdOf, BalanceOf, FundIndex, FundInfoOf};
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, ReservableCurrency, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{traits::Zero, Saturating};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency in which the crowdfunds will be denominated
		type Currency: ReservableCurrency<AccountIdOf<Self>>;

		/// The amount to be held on deposit by the owner of a crowdfund
		type SubmissionDeposit: Get<BalanceOf<Self>>;

		/// The amount to be held on deposit by the owner of a crowdfund
		type MinContribution: Get<BalanceOf<Self>>;

		/// The period of time (in blocks) after an unsuccessful crowdfund ending during which
		/// contributors are able to withdraw their funds. After this period, their funds are lost.
		type RetirementPeriod: Get<BlockNumberFor<Self>>;
	}

	#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, RuntimeDebug)]
	pub struct FundInfo<AccountId, Balance, BlockNumber> {
		/// The account that will receive the funds if the campaign is successful
		pub beneficiary: AccountId,
		/// The amount of deposit placed
		pub deposit: Balance,
		/// The total amount raised
		pub raised: Balance,
		/// Block number after which funding must have succeeded
		pub end: BlockNumber,
		/// Upper bound on `raised`
		pub goal: Balance,
	}

	#[pallet::storage]
	#[pallet::getter(fn funds)]
	pub type Funds<T> = StorageMap<_, Blake2_128Concat, FundIndex, FundInfoOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	pub type FundCount<T> = StorageValue<_, FundIndex, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created(FundIndex, BlockNumberFor<T>),
		Contributed(T::AccountId, FundIndex, BalanceOf<T>, BlockNumberFor<T>),
		Withdrew(T::AccountId, FundIndex, BalanceOf<T>, BlockNumberFor<T>),
		Dissolved(FundIndex, BlockNumberFor<T>, T::AccountId),
		Dispensed(FundIndex, BlockNumberFor<T>, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Crowdfund must end after it starts
		EndTooEarly,
		/// Must contribute at least the minimum amount of funds
		ContributionTooSmall,
		/// The fund index specified does not exist
		InvalidIndex,
		/// The crowdfund's contribution period has ended; no more contributions will be accepted
		ContributionPeriodOver,
		/// You cannot withdraw funds because you have not contributed any
		NoContribution,
		/// You may not withdraw or dispense funds while the fund is still active
		FundStillActive,
		/// You cannot dissolve a fund that has not yet completed its retirement period
		FundNotRetired,
		/// Cannot dispense funds from an unsuccessful fund
		UnsuccessfulFund,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new fund
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn create(
			origin: OriginFor<T>,
			beneficiary: AccountIdOf<T>,
			goal: BalanceOf<T>,
			end: BlockNumberFor<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(end > block_number, Error::<T>::EndTooEarly);

			let deposit = T::SubmissionDeposit::get();
			let imb = T::Currency::withdraw(
				&creator,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;

			let index = FundCount::<T>::get();
			// not protected against overflow, see safemath section
			FundCount::<T>::put(index + 1);

			// No fees are paid here if we need to create this account; that's why we don't just
			// use the stock `transfer`.
			T::Currency::resolve_creating(&Self::fund_account_id(index), imb);

			<Funds<T>>::insert(
				index,
				FundInfo { beneficiary, deposit, raised: Zero::zero(), end, goal },
			);

			Self::deposit_event(Event::Created(index, block_number));
			Ok(())
		}
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn contribute(
			origin: OriginFor<T>,
			index: FundIndex,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);

			let mut fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(fund.end > block_number, Error::<T>::ContributionPeriodOver);

			// Add contribution to the fund
			T::Currency::transfer(
				&who,
				&Self::fund_account_id(index),
				value,
				ExistenceRequirement::AllowDeath,
			)?;
			fund.raised += value;
			Funds::<T>::insert(index, &fund);

			let balance = Self::contribution_get(index, &who);
			let balance = balance.saturating_add(value);
			Self::contribution_put(index, &who, &balance);

			Self::deposit_event(Event::Contributed(who, index, balance, block_number));

			Ok(())
		}

		/// Withdraw full balance of a contributor to a fund
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn withdraw(origin: OriginFor<T>, index: FundIndex) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let mut crowdfund = Self::funds(&index).ok_or(Error::<T>::InvalidIndex)?;
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(crowdfund.end < block_number, Error::<T>::FundStillActive);

			let balance = Self::contribution_get(index, &caller);
			ensure!(balance > Zero::zero(), Error::<T>::NoContribution);

			T::Currency::resolve_creating(
				&caller,
				T::Currency::withdraw(
					&Self::fund_account_id(index),
					balance,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Update storage
			Self::contribution_kill(index, &caller);
			crowdfund.raised = crowdfund.raised.saturating_add(balance);
			<Funds<T>>::insert(index, &crowdfund);

			Self::deposit_event(Event::Withdrew(caller, index, balance, block_number));
			Ok(())
		}

		/// Dissolve an entire crowdfund after its retirement period has expired.
		/// Anyone can call this function, and they are incentivized to do so because
		/// they inherit the deposit.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn dissolve(origin: OriginFor<T>, index: FundIndex) -> DispatchResult {
			let reporter = ensure_signed(origin)?;

			let fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(
				block_number >= fund.end + T::RetirementPeriod::get(),
				Error::<T>::FundNotRetired
			);

			let account = Self::fund_account_id(index);

			// Dissolver collects the deposit and any remaining funds
			let _ = T::Currency::resolve_creating(
				&reporter,
				T::Currency::withdraw(
					&account,
					fund.deposit + fund.raised,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Remove the fund info from storage
			<Funds<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dissolved(index, block_number, reporter));

			Ok(())
		}
		/// Dispense a payment to the beneficiary of a successful crowdfund.
		/// The beneficiary receives the contributed funds and the caller receives
		/// the deposit as a reward to incentivize clearing settled crowdfunds out of storage.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn dispense(origin: OriginFor<T>, index: FundIndex) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let now = frame_system::Pallet::<T>::block_number();

			ensure!(now >= fund.end, Error::<T>::FundStillActive);

			// Check that the fund was actually successful
			ensure!(fund.raised >= fund.goal, Error::<T>::UnsuccessfulFund);

			let account = Self::fund_account_id(index);

			// Beneficiary collects the contributed funds
			let _ = T::Currency::resolve_creating(
				&fund.beneficiary,
				T::Currency::withdraw(
					&account,
					fund.raised,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Caller collects the deposit
			let _ = T::Currency::resolve_creating(
				&caller,
				T::Currency::withdraw(
					&account,
					fund.deposit,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Remove the fund info from storage
			<Funds<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dispensed(index, now, caller));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	fn fund_account_id(index: FundIndex) -> T::AccountId {
		PALLET_ID.into_sub_account_truncating(index)
	}
	/// Find the ID associated with the fund
	///
	/// Each fund stores information about its contributors and their contributions in a child trie
	/// This helper function calculates the id of the associated child trie.
	fn id_from_index(index: FundIndex) -> child::ChildInfo {
		let mut buf: Vec<u8> = Vec::new();
		buf.extend(b"crowdfund");
		buf.extend(&index.to_le_bytes()[..]);

		child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
	}

	/// Record a contribution in the associated child trie.
	fn contribution_put(index: FundIndex, who: &T::AccountId, balance: &BalanceOf<T>) {
		let id = Self::id_from_index(index);
		who.using_encoded(|sl| child::put(&id, sl, balance));
	}

	/// Lookup a contribution in the associated child trie.
	fn contribution_get(index: FundIndex, who: &T::AccountId) -> BalanceOf<T> {
		let id = Self::id_from_index(index);
		who.using_encoded(|sl| child::get_or_default(&id, sl))
	}

	/// Remove a contribution from an associated child trie.
	fn contribution_kill(index: FundIndex, who: &T::AccountId) {
		let id = Self::id_from_index(index);
		who.using_encoded(|sl| child::kill(&id, sl));
	}

	/// Remove the entire record of contributions in the associated child trie in a single
	/// storage write.
	fn crowdfund_kill(index: FundIndex) {
		let id = Self::id_from_index(index);
		// The None here means we aren't setting a limit to how many keys to delete.
		// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
		// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
		let _ = child::clear_storage(&id, None, None);
	}
}
