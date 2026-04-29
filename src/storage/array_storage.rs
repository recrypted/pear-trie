use crate::{ArrayStorage, AtomStorage, Indexable};

impl<A: Indexable<N>, V, const N: usize> AtomStorage<A, V> for ArrayStorage<A, V, N> {
    fn get(&self, atom: &A) -> Option<&V> {
        self.values[atom.index()].as_ref()
    }

    fn get_mut(&mut self, atom: &A) -> Option<&mut V> {
        self.values[atom.index()].as_mut()
    }

    fn insert(&mut self, atom: A, value: V) -> Option<V> {
        self.values[atom.index()].replace(value)
    }

    fn remove(&mut self, atom: &A) -> Option<V> {
        self.values[atom.index()].take()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a A, &'a V)>
    where
        V: 'a,
        A: 'a,
    {
        self.atoms
            .iter()
            .zip(self.values.iter())
            .filter_map(|(a, opt)| opt.as_ref().map(|v| (a, v)))
    }

    fn into_iter(self) -> impl Iterator<Item = (A, V)> {
        self.atoms
            .into_iter()
            .zip(self.values)
            .filter_map(|(a, opt)| opt.map(|v| (a, v)))
    }

    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, atom: A, f: F) -> &mut V
    where
        A: Clone,
    {
        self.values[atom.index()].get_or_insert_with(f)
    }

    fn len(&self) -> usize {
        self.atoms.len()
    }
}
