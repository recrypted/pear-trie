use std::hash::Hash;

use crate::{AtomStorage, storage::HashStorage};

impl<A: Eq + Hash, V> AtomStorage<A, V> for HashStorage<A, V> {
    fn get(&self, atom: &A) -> Option<&V> {
        self.0.get(atom)
    }

    fn get_mut(&mut self, atom: &A) -> Option<&mut V> {
        self.0.get_mut(atom)
    }

    fn insert(&mut self, atom: A, value: V) -> Option<V> {
        self.0.insert(atom, value)
    }

    fn remove(&mut self, atom: &A) -> Option<V> {
        self.0.remove(atom)
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a A, &'a V)>
    where
        V: 'a,
        A: 'a,
    {
        self.0.iter()
    }

    fn into_iter(self) -> impl Iterator<Item = (A, V)> {
        self.0.into_iter()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
