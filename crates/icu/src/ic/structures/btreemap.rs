use crate::ic::structures::DefaultMemory;
use derive_more::{Deref, DerefMut};
use ic_stable_structures::{Storable, btreemap::BTreeMap as WrappedBTreeMap};
use std::ops::RangeBounds;

///
/// BTreeMap
/// a wrapper around BTreeMap that uses the default VirtualMemory
///

#[derive(Deref, DerefMut)]
pub struct BTreeMap<K, V>
where
    K: Storable + Ord + Clone,
    V: Storable + Clone,
{
    data: WrappedBTreeMap<K, V, DefaultMemory>,
}

impl<K, V> BTreeMap<K, V>
where
    K: Storable + Ord + Clone,
    V: Storable + Clone,
{
    #[must_use]
    pub fn init(memory: DefaultMemory) -> Self {
        Self {
            data: WrappedBTreeMap::init(memory),
        }
    }

    /// Returns an iterator over all cloned `(K, V)` pairs.
    pub fn iter_pairs(&self) -> impl Iterator<Item = (K, V)> + '_ {
        self.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
    }

    /// Returns an iterator over a range of cloned `(K, V)` pairs.
    pub fn range_pairs<R>(&self, range: R) -> impl Iterator<Item = (K, V)> + '_
    where
        R: RangeBounds<K>,
    {
        self.range(range)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
    }

    /// clear
    /// the original clear() method in the ic-stable-structures library
    /// couldn't be wrapped as it took ownership, so they made a new one
    pub fn clear(&mut self) {
        self.clear_new();
    }
}
