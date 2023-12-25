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
		MemberJoinGroup(T::AccountId, GroupIndex, u32),
		RemoveMember(T::AccountId),
		RemoveGroup(GroupIndex),
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
			ensure!(Self::is_member(&member), "not a member, can't join group");
			<MemberScore<T>>::insert(&index, &member, score);
			<GroupMembership<T>>::insert(&member, &index);

			Self::deposit_event(Event::MemberJoinGroup(member, index, score));
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn remove_member(origin: OriginFor<T>) -> DispatchResult {
			let member_to_remove = ensure_signed(origin)?;
			ensure!(Self::is_member(&member_to_remove), "not a member, can't remove");
			let group_id = <GroupMembership<T>>::take(&member_to_remove);
			<MemberScore<T>>::remove(group_id, &member_to_remove);

			Self::deposit_event(Event::RemoveMember(member_to_remove));
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn remove_group_score(origin: OriginFor<T>, group: GroupIndex) -> DispatchResult {
			let member = ensure_signed(origin)?;
			let group_id = <GroupMembership<T>>::get(&member);
			ensure!(group_id == group, "member isn't in the group, can't remove it");
			let _ = <MemberScore<T>>::clear_prefix(&group_id, u32::MAX, None);

			Self::deposit_event(Event::RemoveGroup(group_id));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn is_member(who: &T::AccountId) -> bool {
		Self::all_members().contains(who)
	}
}
