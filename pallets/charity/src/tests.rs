use crate::{mock::*, Event};
use frame_support::{
	assert_err, assert_ok,
	traits::{Currency, OnUnbalanced},
};
use frame_system::RawOrigin;

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

#[test]
fn cannot_donate_too_much() {
	new_test_ext().execute_with(|| {
		assert_err!(Charity::donate(RuntimeOrigin::signed(1), 14), "Can't make donation");
	})
}

#[test]
fn imbalances_work() {
	new_test_ext().execute_with(|| {
		let imb_atm = 5;
		let imb = pallet_balances::NegativeImbalance::new(imb_atm);
		Charity::on_nonzero_unbalanced(imb);

		let new_pot_total = imb_atm + Balances::minimum_balance();
		assert_eq!(Charity::pot(), new_pot_total);

		System::assert_last_event(Event::<Test>::ImbalanceAbsorbed(imb_atm, new_pot_total).into());
	})
}

#[test]
fn allocation_works() {
	new_test_ext().execute_with(|| {
		// Charity acquires 10 tokens from user 1
		let donation = 10;
		assert_ok!(Charity::donate(RuntimeOrigin::signed(1), donation));

		// Charity allocates 5 tokens to user 2
		let alloc = 5;
		assert_ok!(Charity::allocate(RawOrigin::Root.into(), 2, alloc));

		// Test that the expected events were emitted
		let events = System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| match e {
				RuntimeEvent::Charity(inner) => Some(inner),
				_ => None,
			})
			.collect::<Vec<_>>();

		let expected_events =
			vec![Event::DonationReceived(1, 10, 11), Event::FundsAllocated(2, 5, 6)];
		assert_eq!(events, expected_events);
	})
}

#[test]
fn cant_allocate_too_much() {
	new_test_ext().execute_with(|| {
		// Charity acquires 10 tokens from user 1
		assert_ok!(Charity::donate(RuntimeOrigin::signed(1), 10));

		// Charity tries to allocates 20 tokens to user 2
		assert_err!(Charity::allocate(RawOrigin::Root.into(), 2, 20), "Can't make allocation");
	})
}
