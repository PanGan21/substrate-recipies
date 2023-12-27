use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok};

#[test]
fn init_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(BasicToken::init(RuntimeOrigin::signed(1)));
		assert_eq!(BasicToken::get_balance(1), 21000000);
	})
}

#[test]
fn cannot_double_init() {
	new_test_ext().execute_with(|| {
		assert_ok!(BasicToken::init(RuntimeOrigin::signed(1)));
		assert_err!(BasicToken::init(RuntimeOrigin::signed(1)), Error::<Test>::AlreadyInitialized);
	})
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(BasicToken::init(RuntimeOrigin::signed(1)));
		assert_ok!(BasicToken::transfer(RuntimeOrigin::signed(1), 2, 100));
		assert_eq!(BasicToken::get_balance(1), 20999900);
		assert_eq!(BasicToken::get_balance(2), 100);
	})
}

#[test]
fn cant_spend_more_than_you_have() {
	new_test_ext().execute_with(|| {
		assert_ok!(BasicToken::init(RuntimeOrigin::signed(1)));
		assert_err!(
			BasicToken::transfer(RuntimeOrigin::signed(1), 2, 21000001),
			Error::<Test>::InsufficientFunds
		);
	})
}
