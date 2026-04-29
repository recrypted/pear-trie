use crate::{AtomStorage, SortedVecStorage};

impl<A: Ord, V> AtomStorage<A, V> for SortedVecStorage<A, V> {
    fn get(&self, atom: &A) -> Option<&V> {
        self.0
            .binary_search_by(|(a, _)| a.cmp(atom))
            .ok()
            .map(|i| &self.0[i].1)
    }

    fn get_mut(&mut self, atom: &A) -> Option<&mut V> {
        match self.0.binary_search_by(|(a, _)| a.cmp(atom)) {
            Ok(i) => Some(&mut self.0[i].1),
            Err(_) => None,
        }
    }

    fn insert(&mut self, atom: A, value: V) -> Option<V> {
        match self.0.binary_search_by(|(a, _)| a.cmp(&atom)) {
            Ok(i) => Some(std::mem::replace(&mut self.0[i].1, value)),
            Err(i) => {
                self.0.insert(i, (atom, value));
                None
            }
        }
    }

    fn remove(&mut self, atom: &A) -> Option<V> {
        match self.0.binary_search_by(|(a, _)| a.cmp(atom)) {
            Ok(i) => Some(self.0.remove(i).1),
            Err(_) => None,
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a A, &'a V)>
    where
        V: 'a,
        A: 'a,
    {
        self.0.iter().map(|(a, v)| (a, v))
    }

    fn into_iter(self) -> impl Iterator<Item = (A, V)> {
        self.0.into_iter()
    }

    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, atom: A, f: F) -> &mut V
    where
        A: Clone,
    {
        match self.0.binary_search_by(|(a, _)| a.cmp(&atom)) {
            Ok(i) => &mut self.0[i].1,
            Err(i) => {
                self.0.insert(i, (atom, f()));
                &mut self.0[i].1
            }
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
