pub trait AtomStorage<A, V>: Default {
    /// Look up the value associated with `atom`.
    fn get(&self, atom: &A) -> Option<&V>;

    fn get_mut(&mut self, atom: &A) -> Option<&mut V>;

    fn insert(&mut self, atom: A, value: V) -> Option<V>;

    fn remove(&mut self, atom: &A) -> Option<V>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Atom: Sized {
    type Storage<V>: AtomStorage<Self, V>;
}

pub trait Indexable<const N: usize> {
    /// Index of `self` in `0..N`.
    fn index(&self) -> usize;
    /// Recover the atom value from its index.
    fn from_index(i: usize) -> Self;
}
