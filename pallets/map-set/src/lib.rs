#![cfg_attr(not(feature = "std"), no_std)]

use account_set::AccountSet;
pub use pallet::*;
use sp_std::collections::btree_set::BTreeSet;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// A maximum number of members. When membership reaches this number, no new members may join.
	pub const MAX_MEMBERS: u32 = 16;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn member)]
	pub type Members<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

	#[pallet::storage]
	pub type MemberCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MemberAdded(T::AccountId),
		MemberRemoved(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyMember,
		NotMember,
		MembershipLimitReached,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn add_member(origin: OriginFor<T>) -> DispatchResult {
			let new_member = ensure_signed(origin)?;
			let member_count = <MemberCount<T>>::get();

			ensure!(member_count < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

			ensure!(!<Members<T>>::contains_key(&new_member), Error::<T>::AlreadyMember);

			<Members<T>>::insert(&new_member, ());
			<MemberCount<T>>::mutate(|m| *m += 1);
			Self::deposit_event(Event::MemberAdded(new_member));
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn remove_member(origin: OriginFor<T>) -> DispatchResult {
			let old_member = ensure_signed(origin)?;

			ensure!(<Members<T>>::contains_key(&old_member), Error::<T>::NotMember);

			<Members<T>>::remove(&old_member);
			<MemberCount<T>>::mutate(|m| *m -= 1);
			Self::deposit_event(Event::MemberRemoved(old_member));
			Ok(())
		}
	}
}

impl<T: Config> AccountSet for Pallet<T> {
	type AccountId = T::AccountId;

	fn accounts() -> BTreeSet<Self::AccountId> {
		<Members<T>>::iter().map(|(acc, _)| acc).collect()
	}
}
