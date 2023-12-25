use crate::{mock::*, Event};
use frame_support::{assert_err, assert_ok};

#[test]
fn init_storage() {
	new_test_ext().execute_with(|| {
		assert_ok!(StorageCache::set_copy(RuntimeOrigin::signed(1), 10));
		assert_eq!(StorageCache::some_copy_value(), 10);

		assert_ok!(StorageCache::set_king(RuntimeOrigin::signed(2)));
		assert_eq!(StorageCache::king_member(), Some(2));

		assert_ok!(StorageCache::mock_add_member(RuntimeOrigin::signed(1)));
		assert_err!(
			StorageCache::mock_add_member(RuntimeOrigin::signed(1)),
			"member is already in group"
		);

		assert!(StorageCache::group_members().contains(&1));
	})
}

#[test]
fn increase_value_errors_on_overflow() {
	new_test_ext().execute_with(|| {
		let num1 = u32::MAX - 9;
		assert_ok!(StorageCache::set_copy(RuntimeOrigin::signed(1), num1));

		assert_err!(
			StorageCache::increase_value_no_cache(RuntimeOrigin::signed(1), 10),
			"addition overflowed1"
		);
		assert_err!(
			StorageCache::increase_value_w_copy(RuntimeOrigin::signed(1), 10),
			"addition overflowed1"
		);

		let num2: u32 = 2147483643;
		assert_ok!(StorageCache::set_copy(RuntimeOrigin::signed(1), num2));
		// test second overflow panic for both methods
		assert_err!(
			StorageCache::increase_value_no_cache(RuntimeOrigin::signed(1), 10),
			"addition overflowed2"
		);
		assert_err!(
			StorageCache::increase_value_w_copy(RuntimeOrigin::signed(1), 10),
			"addition overflowed2"
		);
	})
}

#[test]
fn increase_value_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		assert_ok!(StorageCache::set_copy(RuntimeOrigin::signed(1), 25));

		assert_ok!(StorageCache::increase_value_no_cache(RuntimeOrigin::signed(1), 10));
		System::assert_last_event(Event::InefficientValueChange(60, 5).into());
		assert_eq!(StorageCache::some_copy_value(), 60);

		assert_ok!(StorageCache::increase_value_w_copy(RuntimeOrigin::signed(1), 10));
		System::assert_last_event(Event::BetterValueChange(130, 5).into());
		assert_eq!(StorageCache::some_copy_value(), 130);
	})
}

#[test]
fn swap_king_errs_as_intended() {
	new_test_ext().execute_with(|| {
		assert_ok!(StorageCache::mock_add_member(RuntimeOrigin::signed(1)));
		assert_ok!(StorageCache::set_king(RuntimeOrigin::signed(1)));
		assert_err!(
			StorageCache::swap_king_no_cache(RuntimeOrigin::signed(3)),
			"current king is a member so maintains priority"
		);
		assert_err!(
			StorageCache::swap_king_with_cache(RuntimeOrigin::signed(3)),
			"current king is a member so maintains priority"
		);

		assert_ok!(StorageCache::set_king(RuntimeOrigin::signed(2)));
		assert_err!(
			StorageCache::swap_king_no_cache(RuntimeOrigin::signed(3)),
			"new king is not a member so doesn't get priority"
		);
		assert_err!(
			StorageCache::swap_king_with_cache(RuntimeOrigin::signed(3)),
			"new king is not a member so doesn't get priority"
		);
	})
}

#[test]
fn swap_king_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(StorageCache::mock_add_member(RuntimeOrigin::signed(2)));
		assert_ok!(StorageCache::mock_add_member(RuntimeOrigin::signed(3)));

		assert_ok!(StorageCache::set_king(RuntimeOrigin::signed(1)));
		assert_ok!(StorageCache::swap_king_no_cache(RuntimeOrigin::signed(2)));
		System::assert_last_event(Event::InefficientKingSwap(Some(1), 2).into());
		assert_eq!(StorageCache::king_member(), Some(2));

		assert_ok!(StorageCache::set_king(RuntimeOrigin::signed(1)));
		assert_ok!(StorageCache::swap_king_with_cache(RuntimeOrigin::signed(2)));
		System::assert_last_event(Event::BetterKingSwap(Some(1), 2).into());
		assert_eq!(StorageCache::king_member(), Some(2));
	})
}
