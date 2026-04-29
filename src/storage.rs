pub mod array_storage;
pub mod bit_storage;
pub mod btree_storage;
pub mod byte_sparse_storage;
pub mod hash_storage;
pub mod sorted_vec_storage;

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crate::Indexable;

pub struct BitStorage<V>([Option<V>; 2]);

impl<V> Default for BitStorage<V> {
    fn default() -> Self {
        BitStorage([None, None])
    }
}

pub struct ByteSparseStorage<V>(Vec<(u8, V)>);

impl<V> Default for ByteSparseStorage<V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub struct ArrayStorage<A, V, const N: usize> {
    atoms: [A; N],
    values: [Option<V>; N],
}

impl<A: Indexable<N>, V, const N: usize> Default for ArrayStorage<A, V, N> {
    fn default() -> Self {
        let atoms: [A; N] = std::array::from_fn(A::from_index);
        debug_assert!(atoms.iter().enumerate().all(|(i, a)| a.index() == i));
        Self {
            atoms,
            values: [const { Option::<V>::None }; N],
        }
    }
}

pub struct SortedVecStorage<A: Ord, V>(Vec<(A, V)>);

impl<A: Ord, V> Default for SortedVecStorage<A, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub struct BTreeStorage<A: Ord, V>(BTreeMap<A, V>);

impl<A: Ord, V> Default for BTreeStorage<A, V> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

pub struct HashStorage<A: Eq + Hash, V>(HashMap<A, V>);

impl<A: Eq + Hash, V> Default for HashStorage<A, V> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

mod thread_safety {
    use super::*;
    const fn assert_send<T: Send>() {}
    const fn assert_sync<T: Sync>() {}

    const _: () = {
        assert_send::<BitStorage<u32>>();
        assert_sync::<BitStorage<u32>>();

        assert_send::<ByteSparseStorage<u32>>();
        assert_sync::<ByteSparseStorage<u32>>();

        assert_send::<ArrayStorage<u8, u32, 256>>();
        assert_sync::<ArrayStorage<u8, u32, 256>>();

        assert_send::<SortedVecStorage<u32, u32>>();
        assert_sync::<SortedVecStorage<u32, u32>>();

        assert_send::<BTreeStorage<u32, u32>>();
        assert_sync::<BTreeStorage<u32, u32>>();

        assert_send::<HashStorage<u32, u32>>();
        assert_sync::<HashStorage<u32, u32>>();
    };
}
