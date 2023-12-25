#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

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
	pub const MAX_MEMBERS: usize = 16;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

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
			let mut members = <Members<T>>::get();

			ensure!(members.len() < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

			match members.binary_search(&new_member) {
				Ok(_) => return Err(Error::<T>::AlreadyMember.into()),
				Err(index) => {
					members.insert(index, new_member.clone());
					<Members<T>>::put(members);
					Self::deposit_event(Event::MemberAdded(new_member));
					Ok(())
				},
			}
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn remove_member(origin: OriginFor<T>) -> DispatchResult {
			let old_member = ensure_signed(origin)?;
			let mut members = <Members<T>>::get();

			match members.binary_search(&old_member) {
				Ok(index) => {
					members.remove(index);
					<Members<T>>::put(members);
					Self::deposit_event(Event::MemberRemoved(old_member));
					Ok(())
				},
				Err(_) => return Err(Error::<T>::NotMember.into()),
			}
		}
	}
}
