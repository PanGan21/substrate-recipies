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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	pub type GroupIndex = u32;

	#[pallet::storage]
	#[pallet::getter(fn member_score)]
	pub type MemberScore<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		GroupIndex,
		Blake2_128Concat,
		T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn group_membership)]
	pub type GroupMembership<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, GroupIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_members)]
	pub type AllMembers<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewMember(T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn join_all_members(origin: OriginFor<T>) -> DispatchResult {
			let new_member = ensure_signed(origin)?;
			ensure!(!Self::is_member(&new_member), "already a member, can't join");

			<AllMembers<T>>::append(&new_member);
			Self::deposit_event(Event::NewMember(new_member));
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn join_group(origin: OriginFor<T>, index: GroupIndex, score: u32) -> DispatchResult {
			let member = ensure_signed(origin)?;
			ensure!(!Self::is_member(&member), "not a member, can't join group");

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn is_member(who: &T::AccountId) -> bool {
		Self::all_members().contains(who)
	}
}
