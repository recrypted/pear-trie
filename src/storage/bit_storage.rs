use crate::{AtomStorage, storage::BitStorage};

impl<V> AtomStorage<bool, V> for BitStorage<V> {
    fn get(&self, atom: &bool) -> Option<&V> {
        self.0[*atom as usize].as_ref()
    }

    fn get_mut(&mut self, atom: &bool) -> Option<&mut V> {
        self.0[*atom as usize].as_mut()
    }

    fn insert(&mut self, atom: bool, value: V) -> Option<V> {
        self.0[atom as usize].replace(value)
    }

    fn remove(&mut self, atom: &bool) -> Option<V> {
        self.0[*atom as usize].take()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a bool, &'a V)>
    where
        V: 'a,
    {
        const FALSE: &bool = &false;
        const TRUE: &bool = &true;
        self.0
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|v| (if i == 0 { FALSE } else { TRUE }, v)))
    }

    fn into_iter(self) -> impl Iterator<Item = (bool, V)> {
        self.0
            .into_iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.map(|v| (i != 0, v)))
    }

    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, atom: bool, f: F) -> &mut V
        where
            bool: Clone, {
        self.0[atom as usize].get_or_insert_with(f)
    }

    fn len(&self) -> usize {
        2
    }
}
