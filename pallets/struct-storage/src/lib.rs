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
	pub trait Config: frame_system::Config + pallet_balances::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Encode, Decode, Clone, Default, TypeInfo, PartialEq, RuntimeDebug)]
	pub struct InnerThing<Hash, Balance> {
		pub number: u32,
		pub hash: Hash,
		pub balance: Balance,
	}

	type InnerThingOf<T> =
		InnerThing<<T as frame_system::Config>::Hash, <T as pallet_balances::Config>::Balance>;

	#[derive(Encode, Decode, Clone, Default, TypeInfo, RuntimeDebug, PartialEq)]
	pub struct SuperThing<Hash, Balance> {
		pub super_number: u32,
		pub inner_thing: InnerThing<Hash, Balance>,
	}

	#[pallet::storage]
	#[pallet::getter(fn inner_things_by_numbers)]
	pub type InnerThingsByNumber<T> =
		StorageMap<_, Blake2_128Concat, u32, InnerThingOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn super_things_by_super_numbers)]
	pub type SuperThingsBySuperNumbers<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, SuperThing<T::Hash, T::Balance>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewInnerThing(u32, T::Hash, T::Balance),
		NewSuperThingByExistingInner(u32, u32, T::Hash, T::Balance),
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn insert_inner_thing(
			origin: OriginFor<T>,
			number: u32,
			hash: T::Hash,
			balance: T::Balance,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let inner_thing = InnerThing { number, hash, balance };
			<InnerThingsByNumber<T>>::insert(&number, &inner_thing);
			Self::deposit_event(Event::NewInnerThing(number, hash, balance));
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn insert_super_thing_with_existing_inner(
			origin: OriginFor<T>,
			inner_number: u32,
			super_number: u32,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let inner_thing = Self::inner_things_by_numbers(inner_number);
			let super_thing = SuperThing { super_number, inner_thing: inner_thing.clone() };
			<SuperThingsBySuperNumbers<T>>::insert(super_number, super_thing);
			Self::deposit_event(Event::NewSuperThingByExistingInner(
				super_number,
				inner_thing.number,
				inner_thing.hash,
				inner_thing.balance,
			));
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn insert_super_thing_with_new_inner(
			origin: OriginFor<T>,
			inner_number: u32,
			hash: T::Hash,
			balance: T::Balance,
			super_number: u32,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let inner_thing = InnerThing { number: inner_number, hash, balance };
			<InnerThingsByNumber<T>>::insert(&inner_number, &inner_thing);
			Self::deposit_event(Event::NewInnerThing(inner_number, hash, balance));

			let super_thing = SuperThing { super_number, inner_thing: inner_thing.clone() };
			<SuperThingsBySuperNumbers<T>>::insert(super_number, super_thing);
			Self::deposit_event(Event::NewSuperThingByExistingInner(
				super_number,
				inner_thing.number,
				inner_thing.hash,
				inner_thing.balance,
			));

			Ok(())
		}
	}
}
