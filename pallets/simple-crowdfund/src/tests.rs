use crate::{mock::*, Error, FundInfo};
use frame_support::{assert_err, assert_ok, traits::Hooks};

fn run_to_block(n: u64) {
	while System::block_number() < n {
		SimpleCrowdfund::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number() + 1);
		Balances::on_initialize(System::block_number() + 1);
		SimpleCrowdfund::on_initialize(System::block_number() + 1);
	}
}

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

#[test]
fn contribute_handles_basic_errors() {
	new_test_ext().execute_with(|| {
		// Cannot contribute to non-existing fund
		assert_err!(
			SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 49),
			Error::<Test>::InvalidIndex
		);
		// Cannot contribute below minimum contribution
		assert_err!(
			SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 9),
			Error::<Test>::ContributionTooSmall
		);

		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 101));

		// Move past end date
		run_to_block(10);

		// Cannot contribute to ended fund
		assert_err!(
			SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 49),
			Error::<Test>::ContributionPeriodOver
		);
	})
}

#[test]
fn withdraw_works() {
	new_test_ext().execute_with(|| {
		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		// Transfer fees are taken here
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 100));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(2), 0, 200));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(3), 0, 300));

		// Skip all the way to the end
		// SimpleCrowdfund is unsuccessful 100 + 200 + 300 < 1000
		run_to_block(50);

		// User can withdraw their full balance without fees
		assert_ok!(SimpleCrowdfund::withdraw(RuntimeOrigin::signed(1), 0));
		assert_eq!(Balances::free_balance(1), 999);

		assert_ok!(SimpleCrowdfund::withdraw(RuntimeOrigin::signed(2), 0));
		assert_eq!(Balances::free_balance(2), 2000);

		assert_ok!(SimpleCrowdfund::withdraw(RuntimeOrigin::signed(3), 0));
		assert_eq!(Balances::free_balance(3), 3000);
	})
}

#[test]
fn withdraw_handles_basic_errors() {
	new_test_ext().execute_with(|| {
		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		// Transfer fee is taken here
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 49));
		assert_eq!(Balances::free_balance(1), 950);

		run_to_block(5);

		// Cannot withdraw before fund ends
		assert_err!(
			SimpleCrowdfund::withdraw(RuntimeOrigin::signed(1), 0),
			Error::<Test>::FundStillActive
		);

		// Skip to the retirement period
		// SimpleCrowdfund is unsuccessful 100 + 200 + 300 < 1000
		run_to_block(10);

		// Cannot withdraw if they did not contribute
		assert_err!(
			SimpleCrowdfund::withdraw(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NoContribution
		);
		// Cannot withdraw from a non-existent fund
		assert_err!(
			SimpleCrowdfund::withdraw(RuntimeOrigin::signed(1), 1),
			Error::<Test>::InvalidIndex
		);
	});
}

#[test]
fn dissolve_works() {
	new_test_ext().execute_with(|| {
		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		// Transfer fee is taken here
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 100));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(2), 0, 200));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(3), 0, 300));

		// Skip all the way to the end
		// Crowdfund is unsuccessful 100 + 200 + 300 < 1000
		run_to_block(50);

		// Check initiator's balance.
		assert_eq!(Balances::free_balance(1), 899);
		// Check current funds (contributions + deposit)
		assert_eq!(Balances::free_balance(SimpleCrowdfund::fund_account_id(0)), 601);

		// Account 7 dissolves the crowdfund claiming the remaining funds
		assert_ok!(SimpleCrowdfund::dissolve(RuntimeOrigin::signed(7), 0));

		// Fund account is emptied
		assert_eq!(Balances::free_balance(SimpleCrowdfund::fund_account_id(0)), 0);
		// Dissolver account is rewarded
		assert_eq!(Balances::free_balance(7), 601);

		// Storage trie is removed
		assert_eq!(SimpleCrowdfund::contribution_get(0, &0), 0);
		// Fund storage is removed
		assert_eq!(SimpleCrowdfund::funds(0), None);
	});
}

#[test]
fn dissolve_handles_basic_errors() {
	new_test_ext().execute_with(|| {
		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		// Transfer fee is taken here
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 100));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(2), 0, 200));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(3), 0, 300));

		// Cannot dissolve an invalid fund index
		assert_err!(
			SimpleCrowdfund::dissolve(RuntimeOrigin::signed(1), 1),
			Error::<Test>::InvalidIndex
		);
		// Cannot dissolve an active fund
		assert_err!(
			SimpleCrowdfund::dissolve(RuntimeOrigin::signed(1), 0),
			Error::<Test>::FundNotRetired
		);

		run_to_block(10);

		// Cannot disolve an ended but not yet retired fund
		assert_err!(
			SimpleCrowdfund::dissolve(RuntimeOrigin::signed(1), 0),
			Error::<Test>::FundNotRetired
		);
	})
}

#[test]
fn dispense_works() {
	new_test_ext().execute_with(|| {
		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 20, 1000, 9));
		// Transfer fee is taken here
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 100));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(2), 0, 200));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(3), 0, 300));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(3), 0, 400));

		// Skip to the retirement period
		// Crowdfund is successful 100 + 200 + 300 + 400  >= 1000
		run_to_block(10);

		// Check initiator's balance.
		assert_eq!(Balances::free_balance(1), 899);
		// Check current funds (contributions + deposit)
		assert_eq!(Balances::free_balance(SimpleCrowdfund::fund_account_id(0)), 1001);

		// Account 7 dispenses the crowdfund
		assert_ok!(SimpleCrowdfund::dispense(RuntimeOrigin::signed(7), 0));

		// Fund account is emptied
		assert_eq!(Balances::free_balance(SimpleCrowdfund::fund_account_id(0)), 0);
		// Beneficiary account is funded
		assert_eq!(Balances::free_balance(20), 1000);
		// Dispensor account is rewarded deposit
		assert_eq!(Balances::free_balance(7), 1);

		// Storage trie is removed
		assert_eq!(SimpleCrowdfund::contribution_get(0, &0), 0);
		// Fund storage is removed
		assert_eq!(SimpleCrowdfund::funds(0), None);
	});
}

#[test]
fn dispense_handles_basic_errors() {
	new_test_ext().execute_with(|| {
		// Set up a crowdfund
		assert_ok!(SimpleCrowdfund::create(RuntimeOrigin::signed(1), 2, 1000, 9));
		// Transfer fee is taken here
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(1), 0, 100));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(2), 0, 200));
		assert_ok!(SimpleCrowdfund::contribute(RuntimeOrigin::signed(3), 0, 300));

		// Cannot dispense an invalid fund index
		assert_err!(
			SimpleCrowdfund::dispense(RuntimeOrigin::signed(1), 1),
			Error::<Test>::InvalidIndex
		);
		// Cannot dispense an active fund
		assert_err!(
			SimpleCrowdfund::dispense(RuntimeOrigin::signed(1), 0),
			Error::<Test>::FundStillActive
		);

		// Skip to the retirement period
		// Crowdfund is unsuccessful 100 + 200 + 300 < 1000
		run_to_block(10);

		// Cannot disopens an ended but unsuccessful fund
		assert_err!(
			SimpleCrowdfund::dispense(RuntimeOrigin::signed(1), 0),
			Error::<Test>::UnsuccessfulFund
		);
	});
}
