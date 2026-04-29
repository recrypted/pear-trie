#[cfg(feature = "ip")]
pub mod ip;

use std::borrow::Borrow;

use crate::{Atom, AtomStorage};

/// A prefix tree keyed by sequences of atoms of type `A`, mapping to values
/// of type `V`.
///
/// Each node holds an optional value and a child container chosen by the
/// [`Atom`] impl for `A`. Keys are passed in as iterators of atoms (or
/// anything that borrows as one — most read methods accept `Borrow<A>` items
/// so you can use either owned atoms or references).
///
/// # Example
///
/// ```
/// use pear_trie::Trie;
///
/// let mut t: Trie<u8, u32> = Trie::new();
/// t.insert(b"hello".to_vec(), 1);
/// t.insert(b"help".to_vec(),  2);
///
/// assert_eq!(t.len(), 2);
/// assert!(t.has_prefix(b"hel".iter()));
/// assert_eq!(t.get(b"hello".iter()), Some(&1));
/// ```
pub struct Trie<A: Atom, V> {
    root: Node<A, V>,
    len: usize,
}

/// Internal node type — a value slot plus a child container.
///
/// Exposed because it appears in some public type signatures, but you should
/// not need to construct or inspect `Node` directly.
pub struct Node<A: Atom, V> {
    value: Option<V>,
    children: A::Storage<Box<Node<A, V>>>,
}

impl<A: Atom, V> Default for Node<A, V> {
    fn default() -> Self {
        Self {
            value: None,
            children: A::Storage::default(),
        }
    }
}

impl<A: Atom, V> Default for Trie<A, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Atom, V> Trie<A, V> {
    /// Construct an empty trie.
    pub fn new() -> Self {
        Self {
            root: Node::default(),
            len: 0,
        }
    }

    /// Whether the trie holds no values.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Number of key/value pairs currently stored.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Drop every entry, releasing all node storage.
    pub fn clear(&mut self) {
        self.root = Node::default();
        self.len = 0;
    }

    /// Look up the value at `key`, if any.
    ///
    /// `key` is any iterable of items that borrow as `&A` — both
    /// `slice.iter()` (yielding `&A`) and an owned-atom iterator work.
    pub fn get<I, B>(&self, key: I) -> Option<&V>
    where
        I: IntoIterator<Item = B>,
        B: Borrow<A>,
    {
        let mut node = &self.root;
        for atom in key {
            node = node.children.get(atom.borrow())?;
        }
        node.value.as_ref()
    }

    /// Find the longest stored key that is a prefix of `key`, returning its
    /// depth (number of atoms matched) and value.
    ///
    /// A value at the empty key counts as a match of depth `0`. Returns
    /// `None` only if no stored key is a prefix of the input.
    pub fn longest_prefix_match<I, B>(&self, key: I) -> Option<(usize, &V)>
    where
        I: IntoIterator<Item = B>,
        B: Borrow<A>,
    {
        let mut node = &self.root;
        let mut best = self.root.value.as_ref().map(|v| (0usize, v));
        for (depth, atom) in key.into_iter().enumerate() {
            match node.children.get(atom.borrow()) {
                Some(child) => {
                    node = child;
                    if let Some(v) = &node.value {
                        best = Some((depth + 1, v));
                    }
                }
                None => break,
            }
        }
        best
    }

    /// Iterate over every `(key, value)` pair in the trie.
    ///
    /// Iteration order is whatever DFS the underlying storage produces — do
    /// not rely on a specific order unless you've picked a backend that
    /// guarantees one (e.g. `BTreeStorage` yields atoms in sort order).
    pub fn iter(&self) -> TrieIter<'_, A, V> {
        TrieIter {
            stack: vec![(&self.root, Vec::new())],
        }
    }

    /// Whether `key` has an associated value in the trie.
    pub fn contains_key<I, B>(&self, key: I) -> bool
    where
        I: IntoIterator<Item = B>,
        B: Borrow<A>,
    {
        self.get(key).is_some()
    }

    /// Look up a mutable reference to the value at `key`.
    pub fn get_mut<I, B>(&mut self, key: I) -> Option<&mut V>
    where
        I: IntoIterator<Item = B>,
        B: Borrow<A>,
    {
        let mut node: &mut Node<A, V> = &mut self.root;
        for atom in key {
            let next = node.children.get_mut(atom.borrow())?;
            node = next.as_mut();
        }
        node.value.as_mut()
    }

    /// Remove the value at `key` and prune any nodes that become empty as a
    /// result. Returns the removed value, or `None` if `key` wasn't present.
    ///
    /// This is the recommended remove path: it keeps the trie's memory
    /// footprint tight when keys come and go. See [`vacate`](Self::vacate)
    /// for the non-pruning variant when you expect to re-insert into the
    /// same region soon.
    pub fn remove<I, B>(&mut self, key: I) -> Option<V>
    where
        I: IntoIterator<Item = B>,
        B: Borrow<A>,
    {
        fn descend<A, V, J, B>(node: &mut Node<A, V>, iter: &mut J) -> (Option<V>, bool)
        where
            A: Atom,
            J: Iterator<Item = B>,
            B: Borrow<A>,
        {
            match iter.next() {
                None => {
                    let taken = node.value.take();
                    let prune = taken.is_some() && node.value.is_none() && node.children.is_empty();
                    (taken, prune)
                }
                Some(atom) => {
                    let Some(child) = node.children.get_mut(atom.borrow()) else {
                        return (None, false);
                    };
                    let (taken, prune_child) = descend(child.as_mut(), iter);
                    if prune_child {
                        node.children.remove(atom.borrow());
                    }
                    let prune_self = node.value.is_none() && node.children.is_empty();
                    (taken, prune_self)
                }
            }
        }

        let mut iter = key.into_iter();
        let (taken, _) = descend(&mut self.root, &mut iter);
        if taken.is_some() {
            self.len -= 1;
        }
        taken
    }

    /// Clear the value at `key` without pruning the path that led to it.
    ///
    /// The leaf's value slot becomes empty; intermediate nodes stay in place.
    /// Useful when you expect to re-insert nearby keys soon and want to keep
    /// the path warm. If you're not sure, use [`remove`](Self::remove) — it
    /// has the same return contract and reclaims memory.
    pub fn vacate<B, I>(&mut self, key: I) -> Option<V>
    where
        I: IntoIterator<Item = B>,
        B: Borrow<A>,
    {
        let mut node: &mut Node<A, V> = &mut self.root;
        for atom in key {
            let next = node.children.get_mut(atom.borrow())?;
            node = next.as_mut();
        }

        let prev = node.value.take();
        if prev.is_some() {
            self.len -= 1;
        }
        prev
    }

    /// Whether any stored key starts with `prefix`.
    ///
    /// Returns `true` if `prefix` is itself a stored key, or if any stored
    /// key extends it.
    pub fn has_prefix<I, B>(&self, prefix: I) -> bool
    where
        I: IntoIterator<Item = B>,
        B: Borrow<A>,
    {
        let mut node = &self.root;
        for atom in prefix {
            match node.children.get(atom.borrow()) {
                Some(child) => node = child.as_ref(),
                None => return false,
            }
        }
        node.value.is_some() || !node.children.is_empty()
    }

    /// Iterate over every `(key, value)` pair whose key starts with `prefix`.
    ///
    /// An empty `prefix` yields the full trie. If `prefix` doesn't exist in
    /// the trie at all, the returned iterator is empty.
    pub fn prefix_iter<'a, I>(
        &'a self,
        prefix: I,
    ) -> Box<dyn Iterator<Item = (Vec<&'a A>, &'a V)> + 'a>
    where
        I: IntoIterator<Item = &'a A>,
        A: 'a,
    {
        let prefix_path: Vec<&A> = prefix.into_iter().collect();
        let mut node = &self.root;
        for atom in &prefix_path {
            match node.children.get(atom) {
                Some(child) => node = child.as_ref(),
                None => return Box::new(std::iter::empty()),
            }
        }
        Box::new(subtree_iter(node, prefix_path))
    }
    // remove, get_mut, contains_key, prefix_iter, has_prefix
}

impl<A: Atom + Clone, V> Trie<A, V> {
    /// Insert `value` under `key`, returning the previous value if `key` was
    /// already present.
    pub fn insert<I: IntoIterator<Item = A>>(&mut self, key: I, value: V) -> Option<V> {
        let mut node = &mut self.root;
        for atom in key {
            node = node
                .children
                .get_or_insert_with(atom, || Box::new(Node::default()));
        }
        let prev = node.value.replace(value);
        if prev.is_none() {
            self.len += 1;
        }
        prev
    }
}

fn subtree_iter<'a, A: Atom, V>(
    root: &'a Node<A, V>,
    initial_path: Vec<&'a A>,
) -> impl Iterator<Item = (Vec<&'a A>, &'a V)> + 'a {
    let mut stack = vec![(root, initial_path)];
    std::iter::from_fn(move || {
        while let Some((node, path)) = stack.pop() {
            for (atom, child) in node.children.iter() {
                let mut child_path = path.clone();
                child_path.push(atom);
                stack.push((child.as_ref(), child_path));
            }
            if let Some(v) = &node.value {
                return Some((path, v));
            }
        }
        None
    })
}

/// Borrowing iterator over a [`Trie`], yielding `(Vec<&A>, &V)` pairs.
pub struct TrieIter<'a, A: Atom, V> {
    stack: Vec<(&'a Node<A, V>, Vec<&'a A>)>,
}

impl<'a, A: Atom, V> Iterator for TrieIter<'a, A, V> {
    type Item = (Vec<&'a A>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, path)) = self.stack.pop() {
            for (atom, child) in node.children.iter() {
                let mut child_path = path.clone();
                child_path.push(atom);
                self.stack.push((child.as_ref(), child_path));
            }

            if let Some(v) = &node.value {
                return Some((path, v));
            }
        }
        None
    }
}

/// Owning iterator over a [`Trie`], yielding `(Vec<A>, V)` pairs.
pub struct TrieIntoIter<A: Atom, V> {
    stack: Vec<(Node<A, V>, Vec<A>)>,
}

impl<A: Atom + Clone, V> Iterator for TrieIntoIter<A, V> {
    type Item = (Vec<A>, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, path)) = self.stack.pop() {
            let Node { value, children } = node;
            for (atom, boxed_child) in children.into_iter() {
                let mut child_path = path.clone();
                child_path.push(atom);
                self.stack.push((*boxed_child, child_path));
            }
            if let Some(v) = value {
                return Some((path, v));
            }
        }
        None
    }
}

impl<A: Atom + Clone, V> IntoIterator for Trie<A, V> {
    type Item = (Vec<A>, V);
    type IntoIter = TrieIntoIter<A, V>;

    fn into_iter(self) -> Self::IntoIter {
        TrieIntoIter {
            stack: vec![(self.root, Vec::new())],
        }
    }
}

impl<'a, A: Atom, V> IntoIterator for &'a Trie<A, V> {
    type Item = (Vec<&'a A>, &'a V);
    type IntoIter = TrieIter<'a, A, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A, V, K> FromIterator<(K, V)> for Trie<A, V>
where
    A: Atom + Clone,
    K: IntoIterator<Item = A>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut trie = Self::new();
        trie.extend(iter);
        trie
    }
}

impl<A, V, K> Extend<(K, V)> for Trie<A, V>
where
    A: Atom + Clone,
    K: IntoIterator<Item = A>,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

/// A view into a single entry in a [`Trie`], either occupied or vacant.
///
/// Returned by [`Trie::entry`]. Lets you do conditional insert / update in a
/// single descent — analogous to `HashMap::Entry`.
pub enum Entry<'a, A: Atom + Clone, V> {
    /// The key already has a value associated with it.
    Occupied(OccupiedEntry<'a, A, V>),
    /// The key is not yet associated with a value.
    Vacant(VacantEntry<'a, A, V>),
}

/// A view into an entry that already has a value.
pub struct OccupiedEntry<'a, A: Atom, V> {
    /// invariant: node.value.is_some()
    node: &'a mut Node<A, V>,
    len: &'a mut usize,
}

impl<'a, A: Atom, V> OccupiedEntry<'a, A, V> {
    /// Get a reference to the stored value.
    pub fn get(&self) -> &V {
        self.node.value.as_ref().unwrap()
    }

    /// Get a mutable reference to the stored value.
    pub fn get_mut(&mut self) -> &mut V {
        self.node.value.as_mut().unwrap()
    }

    /// Consume the entry and return a mutable reference tied to the trie's
    /// lifetime.
    pub fn into_mut(self) -> &'a mut V {
        self.node.value.as_mut().unwrap()
    }

    /// Replace the stored value, returning the old one.
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.node.value.as_mut().unwrap(), value)
    }

    /// Remove the value at this slot without pruning the path.
    ///
    /// The structural counterpart of [`Trie::vacate`]: the slot is cleared,
    /// the path stays. To also prune empty branches, drop the entry and call
    /// [`Trie::remove`] instead.
    pub fn vacate(self) -> V {
        *self.len -= 1;
        self.node.value.take().unwrap()
    }
}

impl<'a, A: Atom + Clone, V> VacantEntry<'a, A, V> {
    /// Insert `value` at this slot, creating any missing nodes on the way.
    /// Returns a mutable reference to the new value.
    pub fn insert(self, value: V) -> &'a mut V {
        let VacantEntry {
            mut node,
            remaining,
            len,
        } = self;
        for atom in remaining {
            node = node
                .children
                .get_or_insert_with(atom, || Box::new(Node::default()));
        }
        *len += 1;
        node.value = Some(value);
        node.value.as_mut().unwrap()
    }
}

/// A view into an entry that has no value yet.
///
/// Carries the deepest existing node on the key path and any atoms still to
/// descend. The actual node creation only happens when you call
/// [`insert`](VacantEntry::insert) — taking an entry and dropping it without
/// inserting leaves the trie unchanged.
pub struct VacantEntry<'a, A: Atom + Clone, V> {
    /// deepest existing node on key path
    node: &'a mut Node<A, V>,
    /// atoms still left to descend (or empty)
    remaining: Vec<A>,
    len: &'a mut usize,
}

impl<A: Atom + Clone, V> Trie<A, V> {
    /// Get the [`Entry`] for `key`, descending the trie once.
    ///
    /// Use this when you want to inspect or update without paying for two
    /// lookups (e.g. `if !contains { insert }`). Pairs naturally with
    /// [`or_insert`](Entry::or_insert), [`or_insert_with`](Entry::or_insert_with),
    /// and [`and_modify`](Entry::and_modify).
    pub fn entry<I: IntoIterator<Item = A>>(&mut self, key: I) -> Entry<'_, A, V> {
        let Trie { root, len } = self;
        let mut node = root;
        let mut iter = key.into_iter();

        loop {
            let Some(atom) = iter.next() else { break };
            if node.children.get(&atom).is_none() {
                let mut remaining = Vec::with_capacity(1);
                remaining.push(atom);
                remaining.extend(iter);
                return Entry::Vacant(VacantEntry {
                    node,
                    remaining,
                    len,
                });
            }
            node = node.children.get_mut(&atom).unwrap();
        }

        if node.value.is_some() {
            Entry::Occupied(OccupiedEntry { node, len })
        } else {
            Entry::Vacant(VacantEntry {
                node,
                remaining: Vec::new(),
                len,
            })
        }
    }
}

impl<'a, A: Atom + Clone, V> Entry<'a, A, V> {
    /// Return a mutable reference to the value, inserting `default` if vacant.
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }

    /// Return a mutable reference to the value, inserting `default()` if
    /// vacant. The closure runs only on the vacant path.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(default()),
        }
    }

    /// Like [`or_insert_with`](Self::or_insert_with), but the default closure
    /// receives the atoms that still need to be created (i.e. the suffix of
    /// the key past the deepest existing node).
    pub fn or_insert_with_key<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce(&[A]) -> V,
    {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let value = default(&v.remaining);
                v.insert(value)
            }
        }
    }

    /// Run `f` on the stored value if the entry is occupied; do nothing on
    /// vacant. Returns the entry so it chains with `or_insert*`.
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(ref mut o) = self {
            f(o.get_mut());
        };
        self
    }
}
