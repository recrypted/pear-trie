use crate::{AtomStorage, storage::ByteSparseStorage};

impl<V> AtomStorage<u8, V> for ByteSparseStorage<V> {
    fn get(&self, atom: &u8) -> Option<&V> {
        self.0.iter().find_map(|(a, v)| (a == atom).then_some(v))
    }

    fn get_mut(&mut self, atom: &u8) -> Option<&mut V> {
        self.0
            .iter_mut()
            .find_map(|(a, v)| (a == atom).then_some(v))
    }

    fn insert(&mut self, atom: u8, value: V) -> Option<V> {
        for (a, v) in &mut self.0 {
            if *a == atom {
                return Some(std::mem::replace(v, value));
            }
        }
        self.0.push((atom, value));
        None
    }

    fn remove(&mut self, atom: &u8) -> Option<V> {
        let pos = self.0.iter().position(|(a, _)| a == atom)?;
        Some(self.0.swap_remove(pos).1)
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a u8, &'a V)>
    where
        V: 'a,
        u8: 'a,
    {
        self.0.iter().map(|(a, v)| (a, v))
    }

    fn into_iter(self) -> impl Iterator<Item = (u8, V)> {
        self.0.into_iter()
    }

    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, atom: u8, f: F) -> &mut V
    where
        u8: Clone,
    {
        match self.0.iter().position(|(a, _)| *a == atom) {
            Some(i) => &mut self.0[i].1,
            None => {
                self.0.push((atom, f()));
                &mut self.0.last_mut().unwrap().1
            }
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
