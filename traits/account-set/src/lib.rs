#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::collections::btree_set::BTreeSet;

pub trait AccountSet {
	type AccountId;

	fn accounts() -> BTreeSet<Self::AccountId>;
}
