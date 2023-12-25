use crate::{mock::*, Event};
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
