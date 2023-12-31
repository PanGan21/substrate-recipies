use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_arithmetic::Permill;
use substrate_fixed::types::U16F16;

#[test]
fn all_accumulators_start_at_one() {
	new_test_ext().execute_with(|| {
		assert_eq!(FixedPoint::manual_value(), 1 << 16);
		assert_eq!(FixedPoint::permil_value(), Permill::one());
		assert_eq!(FixedPoint::fixed_value(), 1);
	})
}

#[test]
fn fixed_impl_works() {
	new_test_ext().execute_with(|| {
		// Setup some constants
		let one = U16F16::from_num(1);
		let half = one / 2;
		let quarter = half / 2;

		// Multiply by half
		assert_ok!(FixedPoint::update_fixed(RuntimeOrigin::signed(1), half));

		// Ensure the new value is correct
		assert_eq!(FixedPoint::fixed_value(), half);

		// Multiply by half again
		assert_ok!(FixedPoint::update_fixed(RuntimeOrigin::signed(1), half));

		// Ensure the new value is correct
		assert_eq!(FixedPoint::fixed_value(), quarter);

		// Test that the expected events were emitted
		let our_events = System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| match e {
				RuntimeEvent::FixedPoint(inner) => Some(inner),
				_ => None,
			})
			.collect::<Vec<_>>();

		let expected_events =
			vec![Event::FixedUpdated(half, half), Event::FixedUpdated(half, quarter)];

		assert_eq!(our_events, expected_events);
	})
}

#[test]
fn fixed_impl_overflows() {
	new_test_ext().execute_with(|| {
		// U16F16 has 16 bits of integer storage, so just like with our manual
		// implementation, a value of 2 ^ 17 will cause overflow.

		// Multiply by 2 ^ 10
		assert_ok!(FixedPoint::update_fixed(RuntimeOrigin::signed(1), U16F16::from_num(1 << 10)));

		// Multiple by an additional 2 ^  7 which should cause the overflow
		assert_noop!(
			FixedPoint::update_fixed(RuntimeOrigin::signed(1), U16F16::from_num(1 << 7)),
			Error::<Test>::Overflow
		);
	})
}
