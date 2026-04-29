use crate::{ArrayStorage, BTreeStorage, BitStorage, ByteSparseStorage, Indexable};

use super::Atom;

impl Atom for bool {
    type Storage<V> = BitStorage<V>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DenseByte(pub u8);

impl Indexable<256> for DenseByte {
    fn index(&self) -> usize {
        self.0 as usize
    }

    fn from_index(i: usize) -> Self {
        Self(i as u8)
    }
}

impl Atom for DenseByte {
    type Storage<V> = ArrayStorage<DenseByte, V, 256>;
}

impl Atom for u8 {
    type Storage<V> = ByteSparseStorage<V>;
}

impl Indexable<256> for i8 {
    fn index(&self) -> usize {
        self.cast_unsigned() as usize
    }

    fn from_index(i: usize) -> Self {
        (i as u8).cast_signed()
    }
}

impl Atom for char {
    type Storage<V> = BTreeStorage<char, V>;
}

impl Atom for u16 {
    type Storage<V> = BTreeStorage<u16, V>;
}

impl Atom for u32 {
    type Storage<V> = BTreeStorage<u32, V>;
}

impl Atom for u64 {
    type Storage<V> = BTreeStorage<u64, V>;
}

impl Atom for usize {
    type Storage<V> = BTreeStorage<usize, V>;
}

impl Atom for i16 {
    type Storage<V> = BTreeStorage<i16, V>;
}

impl Atom for i32 {
    type Storage<V> = BTreeStorage<i32, V>;
}

impl Atom for i64 {
    type Storage<V> = BTreeStorage<i64, V>;
}

impl Atom for isize {
    type Storage<V> = BTreeStorage<isize, V>;
}

impl Atom for String {
    type Storage<V> = BTreeStorage<String, V>;
}

impl Atom for &'static str {
    type Storage<V> = BTreeStorage<&'static str, V>;
}
