use crate::{mock::*, Event};
use frame_support::{assert_ok, traits::Hooks};

#[test]
fn deposit_withdraw_discrete_works() {
	new_test_ext().execute_with(|| {
		// Deposit 10 tokens
		assert_ok!(CompoundingInterest::deposit_discrete(RuntimeOrigin::signed(1), 10));

		// Withdraw 5 tokens
		assert_ok!(CompoundingInterest::withdraw_discrete(RuntimeOrigin::signed(1), 5));

		// Test that the expected events were emitted
		let our_events = System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| match e {
				RuntimeEvent::CompoundingInterest(inner) => Some(inner),
				_ => None,
			})
			.collect::<Vec<_>>();

		let expected_events = vec![Event::DepositedDiscrete(10), Event::WithdrewDiscrete(5)];

		assert_eq!(our_events, expected_events);

		// Check that five tokens are still there
		assert_eq!(CompoundingInterest::discrete_account(), 5);
	})
}

#[test]
fn discrete_interest_works() {
	new_test_ext().execute_with(|| {
		// Deposit 100 tokens
		assert_ok!(CompoundingInterest::deposit_discrete(RuntimeOrigin::signed(1), 100));

		// balance should not change after the 3rd block
		CompoundingInterest::on_finalize(3);
		assert_eq!(CompoundingInterest::discrete_account(), 100);

		// on_finalize should compute interest on 10th block
		CompoundingInterest::on_finalize(10);

		// Test that the expected events were emitted
		let our_events = System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| match e {
				RuntimeEvent::CompoundingInterest(inner) => Some(inner),
				_ => None,
			})
			.collect::<Vec<_>>();

		let expected_events =
			vec![Event::DepositedDiscrete(100), Event::DiscreteInterestApplied(50)];

		assert_eq!(our_events, expected_events);

		// Check that the balance has updated
		assert_eq!(CompoundingInterest::discrete_account(), 150);
	})
}
