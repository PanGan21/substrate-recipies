use crate::{mock::*, Event, GroupMembership, MemberScore};
use frame_support::{assert_err, assert_ok};

#[test]
fn join_all_members_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(DoubleMap::join_all_members(RuntimeOrigin::signed(1)));
		assert_err!(
			DoubleMap::join_all_members(RuntimeOrigin::signed(1)),
			"already a member, can't join"
		);
		System::assert_last_event(Event::<Test>::NewMember(1).into());
		assert_eq!(DoubleMap::all_members(), vec![1]);
	})
}

#[test]
fn join_group_works() {
	new_test_ext().execute_with(|| {
		assert_err!(
			DoubleMap::join_group(RuntimeOrigin::signed(1), 3, 5),
			"not a member, can't join group"
		);

		assert_ok!(DoubleMap::join_all_members(RuntimeOrigin::signed(1)));
		assert_ok!(DoubleMap::join_group(RuntimeOrigin::signed(1), 3, 5));
		System::assert_last_event(Event::<Test>::MemberJoinGroup(1, 3, 5).into());
		assert_eq!(DoubleMap::group_membership(1), 3);
		assert_eq!(DoubleMap::member_score(3, 1), 5);
	})
}

#[test]
fn remove_member_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(DoubleMap::join_all_members(RuntimeOrigin::signed(1)));
		assert_ok!(DoubleMap::join_group(RuntimeOrigin::signed(1), 3, 5));
		assert_ok!(DoubleMap::remove_member(RuntimeOrigin::signed(1)));
		System::assert_last_event(Event::<Test>::RemoveMember(1).into());
		assert!(!<GroupMembership<Test>>::contains_key(1));
		assert!(!<MemberScore<Test>>::contains_key(3, 1));
	})
}

#[test]
fn remove_group_score_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(DoubleMap::join_all_members(RuntimeOrigin::signed(1)));
		assert_ok!(DoubleMap::join_all_members(RuntimeOrigin::signed(2)));
		assert_ok!(DoubleMap::join_all_members(RuntimeOrigin::signed(3)));
		assert_ok!(DoubleMap::join_group(RuntimeOrigin::signed(1), 3, 5));
		assert_ok!(DoubleMap::join_group(RuntimeOrigin::signed(2), 3, 5));
		assert_ok!(DoubleMap::join_group(RuntimeOrigin::signed(3), 3, 5));

		assert_err!(
			DoubleMap::remove_group_score(RuntimeOrigin::signed(4), 3),
			"member isn't in the group, can't remove it"
		);

		assert_err!(
			DoubleMap::remove_group_score(RuntimeOrigin::signed(1), 2),
			"member isn't in the group, can't remove it"
		);

		assert_ok!(DoubleMap::remove_group_score(RuntimeOrigin::signed(1), 3));

		System::assert_last_event(Event::<Test>::RemoveGroup(3).into());

		// check: user 1, 2, 3 should no longer in the group
		assert!(!<MemberScore<Test>>::contains_key(3, 1));
		assert!(!<MemberScore<Test>>::contains_key(3, 2));
		assert!(!<MemberScore<Test>>::contains_key(3, 3));
	})
}
