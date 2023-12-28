use crate::{mock::*, Event, FundInfo};
use frame_support::{assert_err, assert_ok};

#[test]
fn basic_setup_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(System::block_number(), 0);
		assert_eq!(SimpleCrowdfund::fund_count(), 0);
		assert_eq!(SimpleCrowdfund::funds(0), None);
		assert_eq!(SimpleCrowdfund::contribution_get(0, &1), 0);
	})
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		// Now try to create a crowdfund campaign
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		assert_eq!(SimpleCrowdfund::fund_count(), 1);
		// This is what the initial `fund_info` should look like
		let fund_info = FundInfo { beneficiary: 2, deposit: 1, raised: 0, end: 9, goal: 1000 };
		assert_eq!(SimpleCrowdfund::funds(0), Some(fund_info));
		// User has deposit removed from their free balance
		assert_eq!(Balances::free_balance(1), 999);
		// Deposit is placed in crowdfund free balance
		assert_eq!(Balances::free_balance(SimpleCrowdfund::fund_account_id(0)), 1);
	})
}

#[test]
fn create_handles_insufficient_balance() {
	new_test_ext().execute_with(|| {
		assert_err!(
			SimpleCrowdfund::create(RuntimeOrigin::signed(1000), 2, 1000, 9),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	})
}

#[test]
fn contribute_works() {
	new_test_ext().execute_with(|| {
		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		assert_eq!(Balances::free_balance(1), 999);
		assert_eq!(Balances::free_balance(SimpleCrowdfund::fund_account_id(0)), 1);

		// No contributions yet
		assert_eq!(SimpleCrowdfund::contribution_get(0, &1), 0);

		// User 1 contributes to their own crowdfund
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 49));
		// User 1 has spent some funds to do this, transfer fees **are** taken
		assert_eq!(Balances::free_balance(&1), 950);
		// Contributions are stored in the trie
		assert_eq!(SimpleCrowdfund::contribution_get(0, &1), 49);
		// Contributions appear in free balance of crowdfund
		assert_eq!(Balances::free_balance(SimpleCrowdfund::fund_account_id(0)), 50);
		// Last contribution time recorded
		assert_eq!(SimpleCrowdfund::funds(0).unwrap().raised, 49);
	})
}
