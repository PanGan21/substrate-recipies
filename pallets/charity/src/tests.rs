use crate::{mock::*, Event};
use frame_support::{assert_err, assert_ok, traits::Currency};

#[test]
fn pot_min_balance_is_set() {
	new_test_ext().execute_with(|| {
		assert_eq!(Charity::pot(), Balances::minimum_balance());
	})
}

#[test]
fn new_test_ext_behaves() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(&1), 13);
	})
}

#[test]
fn donation_works() {
	new_test_ext().execute_with(|| {
		// User 1 donates 10 of his 13 tokens
		let original_balance = Balances::free_balance(&1);
		let donation = 10;
		assert_ok!(Charity::donate(RuntimeOrigin::signed(1), donation));

		// Charity should have 10 tokens
		let new_pot_total = Balances::minimum_balance() + donation;
		assert_eq!(Charity::pot(), new_pot_total);

		// Donor should have 3 remaining
		assert_eq!(Balances::free_balance(&1), original_balance - donation);

		System::assert_last_event(
			Event::<Test>::DonationReceived(1, donation, new_pot_total).into(),
		);
	})
}
