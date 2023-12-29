#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	pub type Caller<T: Config<I>, I: 'static = ()> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		Called(T::AccountId),
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn call_pal(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			// When writing to storage, we supply, not only a configuration T, but also an
			// instance, I.
			<Caller<T, I>>::put(&caller);
			Self::deposit_event(Event::Called(caller));
			Ok(().into())
		}
	}
}
