#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use ringbuffer::{RingBufferTrait, RingBufferTransient};

mod ringbuffer;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	type BufferIndex = u8;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo, RuntimeDebug)]
	pub struct ValueStruct {
		pub integer: i32,
		pub boolean: bool,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_value)]
	pub type BufferMap<T> = StorageMap<_, Blake2_128Concat, BufferIndex, ValueStruct, ValueQuery>;

	#[pallet::type_value]
	pub fn BufferIndexDefaultValue() -> (BufferIndex, BufferIndex) {
		(0, 0)
	}

	#[pallet::storage]
	#[pallet::getter(fn range)]
	pub type BufferRange<T: Config> =
		StorageValue<_, (BufferIndex, BufferIndex), ValueQuery, BufferIndexDefaultValue>;

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		Popped(i32, bool),
		DummyEvent(T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn add_to_queue(origin: OriginFor<T>, integer: i32, boolean: bool) -> DispatchResult {
			let _user = ensure_signed(origin)?;
			let mut queue = Self::queue_transient();
			queue.push(ValueStruct { integer, boolean });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn add_multiple(
			origin: OriginFor<T>,
			integers: Vec<i32>,
			boolean: bool,
		) -> DispatchResult {
			let _user = ensure_signed(origin)?;
			let mut queue = Self::queue_transient();
			for integer in integers {
				queue.push(ValueStruct { integer, boolean });
			}
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn pop_from_queue(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// only a user can pop from the queue
			let _user = ensure_signed(origin)?;

			let mut queue = Self::queue_transient();
			if let Some(ValueStruct { integer, boolean }) = queue.pop() {
				Self::deposit_event(Event::Popped(integer, boolean));
			}

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Constructor function so we don't have to specify the types every time.
	///
	/// Constructs a ringbuffer transient and returns it as a boxed trait object.
	/// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
	fn queue_transient() -> Box<dyn RingBufferTrait<ValueStruct>> {
		Box::new(
			RingBufferTransient::<ValueStruct, pallet::BufferRange<T>, pallet::BufferMap<T>, u8>::new(),
		)
	}
}
