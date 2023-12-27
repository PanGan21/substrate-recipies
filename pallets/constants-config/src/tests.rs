use crate::{mock::*, Event, SingleValue};
use frame_support::{assert_err, assert_ok, traits::Hooks};

#[test]
fn max_added_exceeded_errs() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ConstantsConfig::add_value(RuntimeOrigin::signed(1), 101),
			"value must be <= maximum add amount constant"
		);
	})
}

#[test]
fn overflow_checked() {
	new_test_ext().execute_with(|| {
		let test_num = u32::MAX - 99;
		<SingleValue<Test>>::put(test_num);

		assert_err!(ConstantsConfig::add_value(RuntimeOrigin::signed(1), 100), "Addition overflow");
	})
}

#[test]
fn add_value_works() {
	new_test_ext().execute_with(|| {
		<SingleValue<Test>>::put(10);

		assert_ok!(ConstantsConfig::add_value(RuntimeOrigin::signed(2), 100));
		assert_ok!(ConstantsConfig::add_value(RuntimeOrigin::signed(3), 100));
		assert_ok!(ConstantsConfig::add_value(RuntimeOrigin::signed(4), 100));

		//Test that the expected events were emitted
		let our_events = System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| {
				if let RuntimeEvent::ConstantsConfig(inner) = e {
					Some(inner)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		let expected_events = vec![
			Event::<Test>::Added(10, 100, 110),
			Event::<Test>::Added(110, 100, 210),
			Event::<Test>::Added(210, 100, 310),
		];

		assert_eq!(our_events, expected_events);
	})
}

#[test]
fn on_finalize_clears() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);
		<SingleValue<Test>>::put(10);
		assert_ok!(ConstantsConfig::add_value(RuntimeOrigin::signed(1), 100));

		ConstantsConfig::on_finalize(10);
		System::assert_last_event(Event::<Test>::Cleared(110).into());
		assert_eq!(ConstantsConfig::single_value(), 0);
	})
}
