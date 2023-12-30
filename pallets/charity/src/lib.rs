#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{traits::Currency, PalletId};
pub use pallet::*;
use sp_runtime::traits::AccountIdConversion;

/// Hardcoded pallet ID; used to create the special Pot Account
/// Must be exactly 8 characters long
const PALLET_ID: PalletId = PalletId(*b"Charity!");

type BalanceOf<T> = <<T as Config>::Currency as Currency<T::AccountId>>::Balance<T>;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{BuildGenesisConfig, Currency, ExistenceRequirement},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency type that the charity deals in
		type Currency: Currency<T::AccountId>;
	}

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig;

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig {
		fn build(&self) {
			let _ = T::Currency::make_free_balance_be(
				Self::account_id(),
				T::Currency::minimum_balance(),
			);
		}
	}

	#[pallet::event]
	pub enum Event<T: Config> {
		/// Donor has made a charitable donation to the charity
		DonationReceived(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// An imbalance from elsewhere in the runtime has been absorbed by the Charity
		ImbalanceAbsorbed(BalanceOf<T>, BalanceOf<T>),
		/// Charity has allocated funds to a cause
		FundsAllocated(T::AccountId, BalanceOf<T>, BalanceOf<T>),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn donate(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			T::Currency::transfer(
				&sender,
				&Self::account_id(),
				amount,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::DonationReceived(sender, Self::account_id(), Self::pot()));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The account ID that holds the Charity's funds
	fn account_id() -> T::AccountId {
		PALLET_ID.into_account_truncating()
	}

	/// The Charity's balance
	fn post() -> BalanceOf<T> {
		T::Currency::free_balance(&Self::account_id());
	}
}
