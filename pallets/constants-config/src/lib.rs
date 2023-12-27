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
	use sp_runtime::traits::Zero;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Maximum amount added per invocation
		type MaxAddend: Get<u32>;

		/// Frequency with which the stored value is deleted
		type ClearFrequency: Get<BlockNumberFor<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn single_value)]
	pub type SingleValue<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The value has ben added to. The parameters are
		/// ( initial amount, amount added, final amount)
		Added(u32, u32, u32),
		/// The value has been cleared. The parameter is the value before clearing.
		Cleared(u32),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			if (n % T::ClearFrequency::get()).is_zero() {
				let old_val = Self::single_value();
				<SingleValue<T>>::put(0);
				Self::deposit_event(Event::Cleared(old_val));
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn add_value(origin: OriginFor<T>, val_to_add: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(
				val_to_add <= T::MaxAddend::get(),
				"value must be <= maximum add amount constant"
			);

			let previous_val = Self::single_value();

			let result = match previous_val.checked_add(val_to_add) {
				Some(r) => r,
				None => return Err("Addition overflow")?,
			};

			<SingleValue<T>>::put(result);
			Self::deposit_event(Event::Added(previous_val, val_to_add, result));

			Ok(())
		}
	}
}
