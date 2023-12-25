#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn some_copy_value)]
	pub type SomeCopyValue<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn king_member)]
	pub type KingMember<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn group_members)]
	pub type GroupMembers<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InefficientValueChange(u32, BlockNumberFor<T>),
		BetterValueChange(u32, BlockNumberFor<T>),
		InefficientKingSwap(Option<T::AccountId>, T::AccountId),
		BetterKingSwap(Option<T::AccountId>, T::AccountId),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn increase_value_no_cache(origin: OriginFor<T>, some_value: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let original_call = <SomeCopyValue<T>>::get();
			let some_calculation =
				original_call.checked_add(some_value).ok_or("addition overflowed1")?;

			let unnecessary_call = <SomeCopyValue<T>>::get();
			let another_calculation =
				some_calculation.checked_add(unnecessary_call).ok_or("addition overflowed2")?;

			<SomeCopyValue<T>>::put(another_calculation);
			let now = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::InefficientValueChange(another_calculation, now));
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn increase_value_w_copy(origin: OriginFor<T>, some_value: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let original_call = <SomeCopyValue<T>>::get();
			let some_calculation =
				original_call.checked_add(some_value).ok_or("addition overflowed1")?;

			let another_calculation =
				some_calculation.checked_add(original_call).ok_or("addition overflowed2")?;

			<SomeCopyValue<T>>::put(another_calculation);
			let now = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::BetterValueChange(another_calculation, now));
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn swap_king_no_cache(origin: OriginFor<T>) -> DispatchResult {
			let new_king = ensure_signed(origin)?;
			let existing_king = <KingMember<T>>::get();
			if let Some(king) = existing_king {
				ensure!(!Self::is_member(&king), "current king is a member so maintains priority");
			}

			ensure!(
				Self::is_member(&new_king.clone()),
				"new king is not a member so doesn't get priority"
			);

			let old_king = <KingMember<T>>::get();
			<KingMember<T>>::put(new_king.clone());

			Self::deposit_event(Event::InefficientKingSwap(old_king, new_king));

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn swap_king_with_cache(origin: OriginFor<T>) -> DispatchResult {
			let new_king = ensure_signed(origin)?;
			let existing_king = <KingMember<T>>::get();
			if let Some(king) = existing_king.clone() {
				ensure!(!Self::is_member(&king), "current king is a member so maintains priority");
			}

			ensure!(
				Self::is_member(&new_king.clone()),
				"new king is not a member so doesn't get priority"
			);

			<KingMember<T>>::put(new_king.clone());

			Self::deposit_event(Event::BetterKingSwap(existing_king, new_king));

			Ok(())
		}

		// ---- for testing purposes ----
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn set_copy(origin: OriginFor<T>, val: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			<SomeCopyValue<T>>::put(val);
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn set_king(origin: OriginFor<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			<KingMember<T>>::put(user);
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn mock_add_member(origin: OriginFor<T>) -> DispatchResult {
			let added = ensure_signed(origin)?;
			ensure!(!Self::is_member(&added), "member is already in group");
			<GroupMembers<T>>::append(added);
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn is_member(who: &T::AccountId) -> bool {
		<GroupMembers<T>>::get().contains(who)
	}
}
