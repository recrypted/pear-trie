pub mod array_storage;
pub mod bit_storage;
pub mod btree_storage;
pub mod byte_sparse_storage;
pub mod hash_storage;
pub mod sorted_vec_storage;

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crate::Indexable;

/// Two-slot array storage for binary alphabets.
///
/// **Optimal when:** the atom alphabet has exactly two values that map
/// cleanly to indices 0 and 1 — `bool`, individual IP bits, the two
/// branches of a decision, etc.
///
/// **Why it wins:** access is a single array index, no hashing, no
/// comparison, no allocation per node beyond the two `Option<V>` slots
/// themselves. The whole alphabet is always represented, so the trie
/// shape is fully predictable. Cache behavior is as good as it gets.
///
/// **Don't use when:** the atom has more than two values. There's no
/// graceful degradation — you'd lose data.
///
/// **Examples of types that should pick this:** `bool`, anything that
/// represents a single bit, two-state enums.
pub struct BitStorage<V>([Option<V>; 2]);

impl<V> Default for BitStorage<V> {
    fn default() -> Self {
        BitStorage([None, None])
    }
}

/// Linear-scan vector storage for small / sparse alphabets where the
/// typical node has few children but the full alphabet is large.
///
/// **Optimal when:** the alphabet is small in practice per node (1–16
/// children) even though the type's full range is much larger. The
/// canonical case is `u8` for ASCII / human-readable text — only a
/// handful of byte values appear at any given trie depth.
///
/// **Why it wins:** for `n < ~16` children, a linear scan over a
/// contiguous `Vec` beats both hashing and ordered tree traversal
/// because every comparison is a cache hit. There's no per-bucket
/// allocator overhead and no hash computation. Memory cost per node
/// is just `n * (size_of::<A>() + size_of::<V>())` — far better than
/// `[Option<V>; 256]` (which would waste ~2 KiB per node for `u8`)
/// when most slots are empty.
///
/// **Don't use when:** nodes routinely hold more than ~16 children, or
/// when `A` is large/expensive to compare. At that point a `BTreeMap`
/// or a dense array wins.
///
/// **Examples of types that should pick this:** `u8` for typical text
/// tries, DNA/RNA tries (alphabet of 4), small `enum`s with 4–8
/// variants.
pub struct ByteSparseStorage<V>(Vec<(u8, V)>);

impl<V> Default for ByteSparseStorage<V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Fixed-size indexed-array storage for atoms with a small known alphabet.
///
/// **Optimal when:** the atom has a known fixed alphabet of size `N` and
/// implements [`Indexable<N>`] — i.e. you can map every atom value to a
/// unique index in `0..N` in O(1). Generalizes [`BitStorage`] (`N = 2`)
/// to any small fixed alphabet.
///
/// **Why it wins:** every operation is a single array index. No
/// hashing, no comparison, no allocation per node beyond the array
/// itself. Lookup, insert, and remove are all `O(1)` with the smallest
/// possible constant factor. The atoms are materialized once at
/// construction (via `Indexable::from_index`) so iteration can hand out
/// real `&A` references without per-yield allocation.
///
/// **Don't use when:** the alphabet is large (memory cost is
/// `N * (size_of::<A>() + size_of::<Option<V>>())` per node — bad above
/// `N ~ 256`), the atom doesn't have a natural index, or the alphabet
/// is unbounded. Use [`BTreeStorage`] or [`SortedVecStorage`] instead.
///
/// **Examples of types that should pick this:** hex digits (`N = 16`),
/// nucleotides (`N = 4`), `u8` when nodes are dense (`N = 256`), small
/// fixed enums (state machines, opcode sets).
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

/// Sorted vector storage with binary-search lookup.
///
/// **Optimal when:** the atom is `Ord`, node sizes are in the
/// 16–256 range, and the workload is read-mostly. Sits between
/// [`ByteSparseStorage`] (best for `n < ~16`) and [`BTreeStorage`]
/// (best for `n >> 256` or write-heavy workloads).
///
/// **Why it wins:** uses roughly half the memory of `BTreeMap` (no
/// per-node tree overhead), keeps `O(log n)` lookup via binary search,
/// and has perfect cache locality during the search — the entire run
/// of comparisons walks contiguous memory. `iter()` yields entries in
/// sorted order for free, which is occasionally a useful invariant.
///
/// **Don't use when:** the workload is insert-heavy. Maintaining sort
/// order on insert is `O(n)` because the elements after the insertion
/// point must shift down by one. The same applies to `remove` — you
/// can't use `swap_remove` here because it would break the sort order.
/// At that point [`BTreeStorage`] wins.
///
/// **Examples of types that should pick this:** `Ord` atoms with
/// medium-cardinality nodes, dictionaries built once and queried many
/// times, immutable lookup tables.
pub struct SortedVecStorage<A: Ord, V>(Vec<(A, V)>);

impl<A: Ord, V> Default for SortedVecStorage<A, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Ordered-tree storage backed by `BTreeMap`.
///
/// **Optimal when:** the atom is `Ord` with cheap comparison and the
/// alphabet is too large for direct indexing (more than a few hundred
/// possible values, or unbounded). The default for "I have an `Ord`
/// type and don't know what else to pick."
///
/// **Why it wins over `HashStorage`:** for the small-to-medium node
/// sizes typical in tries (n < ~1000), `Ord` comparison is dramatically
/// cheaper than hashing — a single-instruction `cmp` for primitives
/// vs. running a hash function over the whole atom. `BTreeMap` packs
/// multiple entries per cache line, so the constant factor on
/// `O(log n)` is small. HashMap's `O(1)` only beats this once the
/// hidden hash cost is amortized over many entries, which trie nodes
/// rarely have.
///
/// **Don't use when:** the atom isn't `Ord` (use `HashStorage`), the
/// alphabet is binary (use `BitStorage`), or the atom is `u8` with
/// sparse-typical-node behavior (use `ByteSparseStorage`).
///
/// **Examples of types that should pick this:** `char`, `i32`, `u32`,
/// `i64`, `u64`, `String`, `&'static str`, structs with derived `Ord`.
pub struct BTreeStorage<A: Ord, V>(BTreeMap<A, V>);

impl<A: Ord, V> Default for BTreeStorage<A, V> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

/// Hash-table storage backed by `HashMap`.
///
/// **Optimal when:** the atom is `Eq + Hash` but **not** `Ord` — i.e.
/// no meaningful ordering exists. This is the last-resort fallback,
/// not a default. For most atom types, `BTreeStorage` is faster
/// because trie nodes are small and hashing's constant factor
/// dominates.
///
/// **Why it exists:** some types are inherently hashable but not
/// orderable (NaN-containing floats, `HashSet`-keyed atoms, types
/// where total order would be arbitrary), and the trie still needs
/// to work for them.
///
/// **Don't use when:** any cheaper alternative applies. In particular,
/// don't reach for `HashStorage` just because `HashMap` feels familiar
/// — `BTreeStorage` outperforms it for typical trie node sizes.
///
/// **Examples of types that should pick this:** types implementing
/// `Eq + Hash` but not `Ord`. Rare in practice.
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
