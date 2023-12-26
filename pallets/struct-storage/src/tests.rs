use crate::{mock::*, Event, InnerThing, SuperThing};
use frame_support::assert_ok;
use sp_core::H256;

#[test]
fn insert_inner_works() {
	new_test_ext().execute_with(|| {
		let data = H256::from_low_u64_be(16);
		assert_ok!(StructStorage::insert_inner_thing(RuntimeOrigin::signed(1), 3, data, 7));

		let expected_storage_item = InnerThing { number: 3, hash: data, balance: 7 };
		assert_eq!(StructStorage::inner_things_by_numbers(3), expected_storage_item);
		System::assert_last_event(Event::<Test>::NewInnerThing(3, data, 7).into())
	})
}

#[test]
fn insert_super_thing_with_existing_inner_works() {
	new_test_ext().execute_with(|| {
		let data = H256::from_low_u64_be(16);
		assert_ok!(StructStorage::insert_inner_thing(RuntimeOrigin::signed(1), 3, data, 7));
		assert_ok!(StructStorage::insert_super_thing_with_existing_inner(
			RuntimeOrigin::signed(1),
			3,
			5
		));

		let expected_inner = InnerThing { number: 3, hash: data, balance: 7 };
		assert_eq!(StructStorage::inner_things_by_numbers(3), expected_inner);

		let expected_outer = SuperThing { super_number: 5, inner_thing: expected_inner };
		assert_eq!(StructStorage::super_things_by_super_numbers(5), expected_outer);
		System::assert_last_event(Event::<Test>::NewSuperThingByExistingInner(5, 3, data, 7).into())
	})
}

#[test]
fn insert_super_thing_with_new_inner_works() {
	new_test_ext().execute_with(|| {
		let data = H256::from_low_u64_be(16);
		assert_ok!(StructStorage::insert_super_thing_with_new_inner(
			RuntimeOrigin::signed(1),
			3,
			data,
			7,
			5
		));

		let expected_inner = InnerThing { number: 3, hash: data, balance: 7 };
		assert_eq!(StructStorage::inner_things_by_numbers(3), expected_inner);
		let expected_outer = SuperThing { super_number: 5, inner_thing: expected_inner };
		assert_eq!(StructStorage::super_things_by_super_numbers(5), expected_outer);

		let events = System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| match e {
				RuntimeEvent::StructStorage(inner) => Some(inner),
				_ => None,
			})
			.collect::<Vec<_>>();

		let expected_events = vec![
			Event::NewInnerThing(3, data, 7).into(),
			Event::NewSuperThingByExistingInner(5, 3, data, 7).into(),
		];
		assert_eq!(events, expected_events);
	})
}
