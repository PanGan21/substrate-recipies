use crate::{mock::*, Error, Event, Members, MAX_MEMBERS};
use frame_support::{assert_err, assert_ok};

#[test]
fn add_members_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(MapSet::add_member(RuntimeOrigin::signed(1)));
		System::assert_last_event(Event::MemberAdded(1).into());
		assert!(<Members<Test>>::contains_key(1));
	})
}

#[test]
fn cant_add_duplicate_member() {
	new_test_ext().execute_with(|| {
		assert_ok!(MapSet::add_member(RuntimeOrigin::signed(1)));
		assert_err!(MapSet::add_member(RuntimeOrigin::signed(1)), Error::<Test>::AlreadyMember);
	});
}

#[test]
fn cant_exceed_max_members() {
	new_test_ext().execute_with(|| {
		for i in 0..MAX_MEMBERS {
			assert_ok!(MapSet::add_member(RuntimeOrigin::signed(i.try_into().unwrap())));
		}

		assert_err!(
			MapSet::add_member(RuntimeOrigin::signed((MAX_MEMBERS + 1).try_into().unwrap())),
			Error::<Test>::MembershipLimitReached
		)
	})
}

#[test]
fn remove_member_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(MapSet::add_member(RuntimeOrigin::signed(1)));
		assert_ok!(MapSet::remove_member(RuntimeOrigin::signed(1)));
		System::assert_has_event(Event::MemberRemoved(1).into());
		assert!(!<Members<Test>>::contains_key(1));
	})
}

#[test]
fn remove_member_handles_error() {
	new_test_ext().execute_with(|| {
		assert_err!(MapSet::remove_member(RuntimeOrigin::signed(1)), Error::<Test>::NotMember)
	})
}
