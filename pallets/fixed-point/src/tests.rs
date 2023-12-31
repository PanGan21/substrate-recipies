use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_arithmetic::Permill;

#[test]
fn all_accumulators_start_at_one() {
	new_test_ext().execute_with(|| {
		assert_eq!(FixedPoint::manual_value(), 1 << 16);
		assert_eq!(FixedPoint::permil_value(), Permill::one());
		assert_eq!(FixedPoint::fixed_value(), 1);
	})
}
