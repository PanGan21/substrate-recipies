#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use sp_arithmetic::Percent;
use substrate_fixed::{transcendental::exp, types::I32F32};

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::traits::Zero;
	use substrate_fixed::types::I32F32;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Encode, Decode, Default, TypeInfo)]
	pub struct ContinuousAccountData<BlockNumber> {
		/// The balance of the account after last manual adjustment
		pub principal: I32F32,
		/// The time (block height) at which the balance was last adjusted
		pub deposit_date: BlockNumber,
	}

	#[pallet::storage]
	#[pallet::getter(fn balance_compound)]
	pub type ContinuousAccount<T: Config> =
		StorageValue<_, ContinuousAccountData<BlockNumberFor<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn discrete_account)]
	pub type DiscreteAccount<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T> {
		/// Deposited some balance into the compounding interest account
		DepositedContinuous(u64),
		/// Withdrew some balance from the compounding interest account
		WithdrewContinuous(u64),
		/// Deposited some balance into the discrete interest account
		DepositedDiscrete(u64),
		/// Withdrew some balance from the discrete interest account
		WithdrewDiscrete(u64),
		/// Some interest has been applied to the discrete interest account
		/// The associated data is just the interest amout (not the new or old balance)
		/// This happens every ten blocks
		DiscreteInterestApplied(u64),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			// Apply newly-accrued discrete interest every ten blocks
			if (n % 10u32.into()).is_zero() {
				// Calculate interest Interest = principal * rate * time
				// We can use the `*` operator for multiplying a `Percent` by a u64
				// because `Percent` implements the trait Mul<u64>
				let interest = Self::discrete_interest_rate() * DiscreteAccount::<T>::get() * 10;

				// The following line, although similar, does not work because
				// u64 does not implement the trait Mul<Percent>
				// let interest = DiscreteAccount::get() * Self::discrete_interest_rate() * 10;

				// Update the balance
				let old_balance = DiscreteAccount::<T>::get();
				DiscreteAccount::<T>::put(old_balance + interest);

				// Emit the event
				Self::deposit_event(Event::DiscreteInterestApplied(interest));
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Deposit some funds into the compounding interest account
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn deposit_continuous(origin: OriginFor<T>, val_to_add: u64) -> DispatchResult {
			ensure_signed(origin)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			let old_value = Self::value_of_continuous_account(&current_block);

			// Update storage for compounding account
			ContinuousAccount::<T>::put(ContinuousAccountData {
				principal: old_value + I32F32::from_num(val_to_add),
				deposit_date: current_block,
			});

			// Emit event
			Self::deposit_event(Event::DepositedContinuous(val_to_add));
			Ok(())
		}

		/// Withdraw some funds from the compounding interest account
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn withdraw_continuous(origin: OriginFor<T>, val_to_take: u64) -> DispatchResult {
			ensure_signed(origin)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			let old_value = Self::value_of_continuous_account(&current_block);

			// Update storage for compounding account
			ContinuousAccount::<T>::put(ContinuousAccountData {
				principal: old_value - I32F32::from_num(val_to_take),
				deposit_date: current_block,
			});

			// Emit event
			Self::deposit_event(Event::WithdrewContinuous(val_to_take));
			Ok(())
		}

		/// Deposit some funds into the discrete interest account
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn deposit_discrete(origin: OriginFor<T>, val_to_add: u64) -> DispatchResult {
			ensure_signed(origin)?;

			let old_value = DiscreteAccount::<T>::get();

			// Update storage for discrete account
			DiscreteAccount::<T>::put(old_value + val_to_add);

			// Emit event
			Self::deposit_event(Event::DepositedDiscrete(val_to_add));
			Ok(())
		}

		/// Withdraw some funds from the discrete interest account
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn withdraw_discrete(origin: OriginFor<T>, val_to_take: u64) -> DispatchResult {
			ensure_signed(origin)?;

			let old_value = DiscreteAccount::<T>::get();

			// Update storage for discrete account
			DiscreteAccount::<T>::put(old_value - val_to_take);

			// Emit event
			Self::deposit_event(Event::WithdrewDiscrete(val_to_take));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// A helper function to evaluate the current value of the continuously compounding interest
	/// account
	pub fn value_of_continuous_account(now: &BlockNumberFor<T>) -> I32F32 {
		// Get the old state of the accout
		let ContinuousAccountData { principal, deposit_date } = ContinuousAccount::<T>::get();

		// Calculate the exponential function (lots of type conversion)
		let elapsed_time_block_number = *now - deposit_date;
		let elapsed_time_u32: u32 = TryInto::try_into(elapsed_time_block_number)
			.ok()
			.expect("blockchain will not exceed 2^32 blocks; qed");
		let elapsed_time_i32f32 = I32F32::from_num(elapsed_time_u32);
		let exponent: I32F32 = Self::continuous_interest_rate() * elapsed_time_i32f32;
		let exp_result : I32F32 = exp(exponent)
			.expect("Interest will not overflow account (at least not until the learner has learned enough about fixed point :)");

		// Return the result interest = principal * e ^ (rate * time)
		principal * exp_result
	}

	/// A helper function to return the hard-coded 5% interest rate
	pub fn discrete_interest_rate() -> Percent {
		Percent::from_percent(5)
	}

	/// A helper function to return the hard-coded 5% interest rate
	pub fn continuous_interest_rate() -> I32F32 {
		// 1 / 20 is 5%. Same interest rate as the discrete account, but in the
		// fancy substrate-fixed format. This I32F32 type represents a 32 bit
		// signed integer where all 32 bits are fractional.
		I32F32::from_num(1) / 10
	}
}
