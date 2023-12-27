use crate::{mock::*, Event, ValueStruct};
use frame_support::assert_ok;

#[test]
fn add_to_queue_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(RingBuffer::add_to_queue(RuntimeOrigin::signed(1), 1, true));
		assert_eq!(RingBuffer::get_value(0), ValueStruct { integer: 1, boolean: true });
		assert_eq!(RingBuffer::range(), (0, 1));
	})
}

#[test]
fn add_multiple_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(RingBuffer::add_multiple(RuntimeOrigin::signed(1), vec![1, 2, 3], true));
		assert_eq!(RingBuffer::get_value(0), ValueStruct { integer: 1, boolean: true });
		assert_eq!(RingBuffer::range(), (0, 3));
	})
}

#[test]
fn pop_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(RingBuffer::add_to_queue(RuntimeOrigin::signed(1), 1, true));
		assert_eq!(RingBuffer::get_value(0), ValueStruct { integer: 1, boolean: true });
		assert_eq!(RingBuffer::range(), (0, 1));

		assert_ok!(RingBuffer::pop_from_queue(RuntimeOrigin::signed(1)));
		assert_eq!(RingBuffer::range(), (1, 1));

		System::assert_last_event(Event::<Test>::Popped(1, true).into());
	})
}
