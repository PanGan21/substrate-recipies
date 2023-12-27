use core::marker::PhantomData;

use codec::{Codec, EncodeLike};
use frame_support::{StorageMap, StorageValue};

/// Trait object presenting the ringbuffer interface.
pub trait RingBufferTrait<Item>
where
	Item: Codec + EncodeLike,
{
	/// Store all changes made in the underlying storage.
	///
	/// Data is not guaranteed to be consistent before this call.
	///
	/// Implementation note: Call in `drop` to increase ergonomics.
	fn commit(&self);
	/// Push an item onto the end of the queue.
	fn push(&mut self, i: Item);
	/// Pop an item from the start of the queue.
	///
	/// Returns `None` if the queue is empty.
	fn pop(&mut self) -> Option<Item>;
	/// Return whether the queue is empty.
	fn is_empty(&self) -> bool;
}

// There is no equivalent trait in std so we create one.
pub trait WrappingOps {
	fn wrapping_add(self, rhs: Self) -> Self;
	fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_wrapping_ops {
	($type:ty) => {
		impl WrappingOps for $type {
			fn wrapping_add(self, rhs: Self) -> Self {
				self.wrapping_add(rhs)
			}

			fn wrapping_sub(self, rhs: Self) -> Self {
				self.wrapping_sub(rhs)
			}
		}
	};
}

impl_wrapping_ops!(u8);
impl_wrapping_ops!(u16);
impl_wrapping_ops!(u32);
impl_wrapping_ops!(u64);

type DefaultIdx = u16;
/// Transient backing data that is the backbone of the trait object.
pub struct RingBufferTransient<Item, B, M, Index = DefaultIdx>
where
	Item: Codec + EncodeLike,
	B: StorageValue<(Index, Index), Query = (Index, Index)>,
	M: StorageMap<Index, Item, Query = Item>,
	Index: Codec + EncodeLike + Copy + Eq + WrappingOps + From<u8>,
{
	start: Index,
	end: Index,
	_phantom: PhantomData<(Item, B, M)>,
}

impl<Item, B, M, Index> RingBufferTransient<Item, B, M, Index>
where
	Item: Codec + EncodeLike,
	B: StorageValue<(Index, Index), Query = (Index, Index)>,
	M: StorageMap<Index, Item, Query = Item>,
	Index: Codec + EncodeLike + Copy + Eq + WrappingOps + From<u8>,
{
	pub fn new() -> RingBufferTransient<Item, B, M, Index> {
		let (start, end) = B::get();
		RingBufferTransient { start, end, _phantom: PhantomData }
	}
}

impl<Item, B, M, Index> Drop for RingBufferTransient<Item, B, M, Index>
where
	Item: Codec + EncodeLike,
	B: StorageValue<(Index, Index), Query = (Index, Index)>,
	M: StorageMap<Index, Item, Query = Item>,
	Index: Codec + EncodeLike + Copy + Eq + WrappingOps + From<u8>,
{
	fn drop(&mut self) {
		Self::commit(self);
	}
}

impl<Item, B, M, Index> RingBufferTrait<Item> for RingBufferTransient<Item, B, M, Index>
where
	Item: Codec + EncodeLike,
	B: StorageValue<(Index, Index), Query = (Index, Index)>,
	M: StorageMap<Index, Item, Query = Item>,
	Index: Codec + EncodeLike + Copy + Eq + WrappingOps + From<u8>,
{
	fn commit(&self) {
		B::put((self.start, self.end))
	}

	fn push(&mut self, i: Item) {
		M::insert(self.end, i);
		// this will intentionally overflow and wrap around when bonds_end
		// reaches `Index::max_value` because we want a ringbuffer.
		let next_index = self.end.wrapping_add(1.into());
		if next_index == self.start {
			// queue presents as empty but is not
			// --> overwrite the oldest item in the FIFO ringbuffer
			self.start = self.start.wrapping_add(1.into());
		}
		self.end = next_index;
	}

	fn pop(&mut self) -> Option<Item> {
		if self.is_empty() {
			return None;
		}
		let item = M::take(self.start);
		self.start = self.start.wrapping_add(1.into());

		Some(item)
	}

	fn is_empty(&self) -> bool {
		self.start == self.end
	}
}

#[cfg(test)]
mod tests {
	use self::pallet_ringbuffer::SomeStruct;

	use super::*;
	use frame_support::traits::{ConstU16, ConstU64};
	use sp_core::H256;
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup},
		BuildStorage,
	};
	use RingBufferTrait;

	#[allow(unused)]
	#[frame_support::pallet(dev_mode)]
	mod pallet_ringbuffer {
		use frame_support::pallet_prelude::*;

		#[pallet::pallet]
		pub struct Pallet<T>(_);

		pub type TestIdx = u8;

		#[derive(Clone, PartialEq, Encode, Decode, Default, Debug, TypeInfo)]
		pub struct SomeStruct {
			pub foo: u64,
			pub bar: u64,
		}

		#[pallet::storage]
		#[pallet::getter(fn get_test_range)]
		pub type TestRange<T> = StorageValue<_, (TestIdx, TestIdx), ValueQuery>;

		#[pallet::storage]
		#[pallet::getter(fn get_test_value)]
		pub type TestMap<T> = StorageMap<_, Twox64Concat, TestIdx, SomeStruct, ValueQuery>;

		#[pallet::config]
		pub trait Config: frame_system::Config {}
	}

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			RingBufferPallet: pallet_ringbuffer,
		}
	);

	type Block = frame_system::mocking::MockBlock<Test>;

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Nonce = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Block = Block;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = ConstU64<250>;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ConstU16<42>;
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl pallet_ringbuffer::Config for Test {}

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	// Trait object that we will be interacting with.
	type RingBuffer = dyn RingBufferTrait<SomeStruct>;
	// Implementation that we will instantiate.
	type Transient = RingBufferTransient<
		SomeStruct,
		pallet_ringbuffer::TestRange<Test>,
		pallet_ringbuffer::TestMap<Test>,
		pallet_ringbuffer::TestIdx,
	>;

	#[test]
	fn simple_push() {
		new_test_ext().execute_with(|| {
			let mut ring: Box<RingBuffer> = Box::new(Transient::new());
			ring.push(pallet_ringbuffer::SomeStruct { foo: 1, bar: 2 });
			ring.commit();
			let start_end = RingBufferPallet::get_test_range();
			assert_eq!(start_end, (0, 1));
			let some_struct = RingBufferPallet::get_test_value(0);
			assert_eq!(some_struct, pallet_ringbuffer::SomeStruct { foo: 1, bar: 2 });
		})
	}

	#[test]
	fn drop_does_commit() {
		new_test_ext().execute_with(|| {
			{
				let mut ring: Box<RingBuffer> = Box::new(Transient::new());
				ring.push(pallet_ringbuffer::SomeStruct { foo: 1, bar: 2 });
			}
			let start_end = RingBufferPallet::get_test_range();
			assert_eq!(start_end, (0, 1));
			let some_struct = RingBufferPallet::get_test_value(0);
			assert_eq!(some_struct, pallet_ringbuffer::SomeStruct { foo: 1, bar: 2 });
		})
	}

	#[test]
	fn simple_pop() {
		new_test_ext().execute_with(|| {
			let mut ring: Box<RingBuffer> = Box::new(Transient::new());

			for i in 1..(pallet_ringbuffer::TestIdx::max_value() as u64) + 2 {
				ring.push(pallet_ringbuffer::SomeStruct { foo: 42, bar: i });
			}
			ring.commit();
			let start_end = RingBufferPallet::get_test_range();
			assert_eq!(
				start_end,
				(1, 0),
				"range should be inverted because the index wrapped around"
			);

			let item = ring.pop();
			ring.commit();
			let (start, end) = RingBufferPallet::get_test_range();
			assert_eq!(start..end, 2..0);
			let item = item.expect("an item should be returned");
			assert_eq!(item.bar, 2, "the struct for field `bar = 2`, was placed at index 1");

			let item = ring.pop();
			ring.commit();
			let (start, end) = RingBufferPallet::get_test_range();
			assert_eq!(start..end, 3..0);
			let item = item.expect("an item should be returned");
			assert_eq!(item.bar, 3, "the struct for field `bar = 3`, was placed at index 2");

			for i in 1..4 {
				ring.push(pallet_ringbuffer::SomeStruct { foo: 21, bar: i });
			}
			ring.commit();
			let start_end = RingBufferPallet::get_test_range();
			assert_eq!(start_end, (4, 3));
		})
	}
}
