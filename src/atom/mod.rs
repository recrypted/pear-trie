mod defaults;
pub use defaults::DenseByte;

/// The per-node child storage used by a [`Trie`](crate::Trie).
///
/// One implementation lives in the trie at every node, holding the children
/// indexed by atom. Implementations are free to pick whatever data structure
/// fits the atom type best — see the storage backends in this crate for the
/// options that ship by default.
pub trait AtomStorage<A, V>: Default {
    /// Look up the value associated with `atom`.
    fn get(&self, atom: &A) -> Option<&V>;

    /// Look up a mutable reference to the value associated with `atom`.
    fn get_mut(&mut self, atom: &A) -> Option<&mut V>;

    /// Insert `value` under `atom`, returning the previous value if any.
    fn insert(&mut self, atom: A, value: V) -> Option<V>;

    /// Remove the entry for `atom`, returning its value if it was present.
    fn remove(&mut self, atom: &A) -> Option<V>;

    /// Iterate over `(&atom, &value)` pairs in storage order.
    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a A, &'a V)>
    where
        V: 'a,
        A: 'a;

    /// Consume the storage, yielding `(atom, value)` pairs.
    fn into_iter(self) -> impl Iterator<Item = (A, V)>;

    /// Number of entries currently stored.
    fn len(&self) -> usize;

    /// Whether this storage holds no entries.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return a mutable reference to the value at `atom`, inserting `f()` if
    /// no entry exists yet.
    ///
    /// The default implementation does a redundant lookup after insertion;
    /// backends should override it whenever they can collapse the work into a
    /// single descent (e.g. `BTreeMap::entry`, indexed array access).
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, atom: A, f: F) -> &mut V
    where
        A: Clone,
    {
        if self.get(&atom).is_none() {
            self.insert(atom.clone(), f());
        }
        self.get_mut(&atom).unwrap()
    }
}

/// A type usable as an atom in a [`Trie`](crate::Trie) key sequence.
///
/// The associated [`Storage`](Atom::Storage) type picks the per-node child
/// container. The crate provides sensible defaults for primitives, `String`,
/// and `&'static str`; implement this trait for your own types when you want
/// a different strategy.
pub trait Atom: Sized {
    /// The per-node storage used to hold children keyed by this atom.
    type Storage<V>: AtomStorage<Self, V>;
}

/// Bijection between an atom type and `0..N`.
///
/// Used by [`ArrayStorage`](crate::ArrayStorage) to map atoms to slots in a
/// fixed-size array. The mapping must be total and round-trip correctly:
/// `Self::from_index(self.index()) == self` for every value.
pub trait Indexable<const N: usize> {
    /// Index of `self` in `0..N`.
    fn index(&self) -> usize;
    /// Recover the atom value from its index.
    fn from_index(i: usize) -> Self;
}
