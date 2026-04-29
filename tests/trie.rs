use pear_trie::{Entry, Trie};

fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

#[test]
fn new_is_empty() {
    let t: Trie<u8, i32> = Trie::new();
    assert!(t.is_empty());
    assert_eq!(t.len(), 0);
}

#[test]
fn insert_and_get() {
    let mut t: Trie<u8, i32> = Trie::new();
    assert_eq!(t.insert(bytes("foo"), 1), None);
    assert_eq!(t.insert(bytes("bar"), 2), None);
    assert_eq!(t.len(), 2);
    assert!(!t.is_empty());

    assert_eq!(t.get(bytes("foo").iter()), Some(&1));
    assert_eq!(t.get(bytes("bar").iter()), Some(&2));
    assert_eq!(t.get(bytes("baz").iter()), None);
    assert_eq!(t.get(bytes("fo").iter()), None);
    assert_eq!(t.get(bytes("foobar").iter()), None);
}

#[test]
fn insert_replaces_returns_old() {
    let mut t: Trie<u8, &'static str> = Trie::new();
    assert_eq!(t.insert(bytes("k"), "a"), None);
    assert_eq!(t.insert(bytes("k"), "b"), Some("a"));
    assert_eq!(t.len(), 1);
    assert_eq!(t.get(bytes("k").iter()), Some(&"b"));
}

#[test]
fn empty_key_inserts_at_root() {
    let mut t: Trie<u8, i32> = Trie::new();
    let empty: Vec<u8> = vec![];
    assert_eq!(t.insert(empty.clone(), 42), None);
    assert_eq!(t.get(empty.iter()), Some(&42));
    assert_eq!(t.len(), 1);
}

#[test]
fn contains_key() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("hello"), 1);
    assert!(t.contains_key(bytes("hello").iter()));
    assert!(!t.contains_key(bytes("hell").iter()));
    assert!(!t.contains_key(bytes("helloo").iter()));
}

#[test]
fn get_mut_modifies() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("a"), 10);
    *t.get_mut(bytes("a").iter()).unwrap() = 99;
    assert_eq!(t.get(bytes("a").iter()), Some(&99));
    assert!(t.get_mut(bytes("missing").iter()).is_none());
}

#[test]
fn remove_returns_value_and_decrements() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("foo"), 1);
    t.insert(bytes("foobar"), 2);
    assert_eq!(t.len(), 2);

    assert_eq!(t.vacate(bytes("foo").iter()), Some(1));
    assert_eq!(t.len(), 1);
    assert_eq!(t.get(bytes("foo").iter()), None);
    // longer key still present
    assert_eq!(t.get(bytes("foobar").iter()), Some(&2));

    // removing missing key returns None and doesn't change len
    assert_eq!(t.vacate(bytes("foo").iter()), None);
    assert_eq!(t.len(), 1);
}

#[test]
fn remove_and_prune_removes_empty_branches() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("foobar"), 1);
    t.insert(bytes("foobaz"), 2);

    assert_eq!(t.remove(bytes("foobar").iter()), Some(1));
    assert_eq!(t.len(), 1);
    assert_eq!(t.get(bytes("foobaz").iter()), Some(&2));

    assert_eq!(t.remove(bytes("foobaz").iter()), Some(2));
    assert!(t.is_empty());
    // The whole branch should now be gone:
    assert!(!t.has_prefix(bytes("foo").iter()));
}

#[test]
fn has_prefix() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("apple"), 1);
    t.insert(bytes("application"), 2);

    assert!(t.has_prefix(bytes("app").iter()));
    assert!(t.has_prefix(bytes("apple").iter()));
    assert!(t.has_prefix(bytes("appl").iter()));
    assert!(!t.has_prefix(bytes("banana").iter()));
    assert!(!t.has_prefix(bytes("applez").iter()));
}

#[test]
fn longest_prefix_match() {
    let mut t: Trie<u8, &'static str> = Trie::new();
    t.insert(bytes("a"), "A");
    t.insert(bytes("ab"), "AB");
    t.insert(bytes("abcd"), "ABCD");

    let key = bytes("abcdefg");
    let (depth, val) = t.longest_prefix_match(key.iter()).unwrap();
    assert_eq!(depth, 4);
    assert_eq!(val, &"ABCD");

    // Falls back to a shorter prefix
    let key = bytes("abc");
    let (depth, val) = t.longest_prefix_match(key.iter()).unwrap();
    assert_eq!(depth, 2);
    assert_eq!(val, &"AB");

    // No match at all
    let key = bytes("zz");
    assert!(t.longest_prefix_match(key.iter()).is_none());
}

#[test]
fn longest_prefix_match_with_root_value() {
    let mut t: Trie<u8, &'static str> = Trie::new();
    t.insert(Vec::<u8>::new(), "ROOT");
    t.insert(bytes("ab"), "AB");

    let (depth, val) = t.longest_prefix_match(bytes("zzz").iter()).unwrap();
    assert_eq!(depth, 0);
    assert_eq!(val, &"ROOT");

    let (depth, val) = t.longest_prefix_match(bytes("ab").iter()).unwrap();
    assert_eq!(depth, 2);
    assert_eq!(val, &"AB");
}

#[test]
fn iter_yields_all_entries() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("a"), 1);
    t.insert(bytes("ab"), 2);
    t.insert(bytes("abc"), 3);
    t.insert(bytes("xy"), 4);

    let mut got: Vec<(Vec<u8>, i32)> = t
        .iter()
        .map(|(k, v)| (k.iter().map(|b| **b).collect(), *v))
        .collect();
    got.sort();

    let mut expected = vec![
        (bytes("a"), 1),
        (bytes("ab"), 2),
        (bytes("abc"), 3),
        (bytes("xy"), 4),
    ];
    expected.sort();
    assert_eq!(got, expected);
}

#[test]
fn prefix_iter() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("apple"), 1);
    t.insert(bytes("apply"), 2);
    t.insert(bytes("apricot"), 3);
    t.insert(bytes("banana"), 4);

    let mut got: Vec<Vec<u8>> = t
        .prefix_iter(bytes("app").iter())
        .map(|(k, _)| k.iter().map(|b| **b).collect())
        .collect();
    got.sort();
    assert_eq!(got, vec![bytes("apple"), bytes("apply")]);

    // No match -> empty iter
    let zzz = bytes("zzz");
    let none: Vec<_> = t.prefix_iter(zzz.iter()).collect();
    assert!(none.is_empty());

    // Empty prefix -> all entries
    let all: Vec<_> = t.prefix_iter(std::iter::empty()).collect();
    assert_eq!(all.len(), 4);
}

#[test]
fn clear_resets() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("a"), 1);
    t.insert(bytes("b"), 2);
    t.clear();
    assert!(t.is_empty());
    assert_eq!(t.len(), 0);
    assert_eq!(t.get(bytes("a").iter()), None);
}

#[test]
fn from_iter_and_extend() {
    let entries = vec![(bytes("a"), 1), (bytes("ab"), 2), (bytes("ac"), 3)];
    let t: Trie<u8, i32> = entries.into_iter().collect();
    assert_eq!(t.len(), 3);
    assert_eq!(t.get(bytes("ab").iter()), Some(&2));

    let mut t2: Trie<u8, i32> = Trie::new();
    t2.extend(vec![(bytes("x"), 10), (bytes("y"), 20)]);
    assert_eq!(t2.len(), 2);
    assert_eq!(t2.get(bytes("y").iter()), Some(&20));
}

#[test]
fn into_iter_consumes() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("a"), 1);
    t.insert(bytes("ab"), 2);

    let mut got: Vec<(Vec<u8>, i32)> = t.into_iter().collect();
    got.sort();
    let mut expected = vec![(bytes("a"), 1), (bytes("ab"), 2)];
    expected.sort();
    assert_eq!(got, expected);
}

#[test]
fn works_with_char_atoms() {
    let mut t: Trie<char, i32> = Trie::new();
    let key: Vec<char> = "héllo".chars().collect();
    t.insert(key.clone(), 7);
    assert_eq!(t.get(key.iter()), Some(&7));
    assert_eq!(t.len(), 1);
}

#[test]
fn works_with_bool_atoms() {
    let mut t: Trie<bool, &'static str> = Trie::new();
    t.insert(vec![true, false, true], "tft");
    t.insert(vec![true, false], "tf");
    assert_eq!(t.get([true, false, true].iter()), Some(&"tft"));
    assert_eq!(t.get([true, false].iter()), Some(&"tf"));
    assert_eq!(t.get([false].iter()), None);

    let (depth, v) = t
        .longest_prefix_match([true, false, true, true].iter())
        .unwrap();
    assert_eq!(depth, 3);
    assert_eq!(v, &"tft");
}

#[test]
fn entry_vacant_or_insert_creates_path() {
    let mut t: Trie<u8, i32> = Trie::new();
    let v = t.entry(bytes("foo")).or_insert(7);
    assert_eq!(*v, 7);
    assert_eq!(t.len(), 1);
    assert_eq!(t.get(bytes("foo").iter()), Some(&7));
}

#[test]
fn entry_occupied_or_insert_returns_existing() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("foo"), 1);

    // or_insert on occupied does NOT replace
    let v = t.entry(bytes("foo")).or_insert(99);
    assert_eq!(*v, 1);
    assert_eq!(t.len(), 1);
}

#[test]
fn entry_or_insert_with_only_runs_default_on_vacant() {
    use std::cell::Cell;
    let calls = Cell::new(0);
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("hit"), 1);

    t.entry(bytes("hit")).or_insert_with(|| {
        calls.set(calls.get() + 1);
        100
    });
    assert_eq!(calls.get(), 0);

    t.entry(bytes("miss")).or_insert_with(|| {
        calls.set(calls.get() + 1);
        100
    });
    assert_eq!(calls.get(), 1);
    assert_eq!(t.get(bytes("miss").iter()), Some(&100));
}

#[test]
fn entry_or_insert_with_key_sees_remaining_atoms() {
    let mut t: Trie<u8, usize> = Trie::new();
    t.insert(bytes("ab"), 0); // ensures "abcde" descent reuses 'a' and 'b'

    let v = t
        .entry(bytes("abcde"))
        .or_insert_with_key(|remaining| remaining.len());
    // "ab" exists, so "cde" is what's left to create
    assert_eq!(*v, 3);
}

#[test]
fn entry_and_modify_then_or_insert() {
    let mut t: Trie<u8, i32> = Trie::new();

    t.entry(bytes("k")).and_modify(|v| *v += 10).or_insert(1);
    assert_eq!(t.get(bytes("k").iter()), Some(&1));

    t.entry(bytes("k")).and_modify(|v| *v += 10).or_insert(1);
    assert_eq!(t.get(bytes("k").iter()), Some(&11));
}

#[test]
fn entry_occupied_get_get_mut_insert() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("x"), 5);

    if let Entry::Occupied(mut o) = t.entry(bytes("x")) {
        assert_eq!(o.get(), &5);
        *o.get_mut() = 6;
        let prev = o.insert(7);
        assert_eq!(prev, 6);
    } else {
        panic!("expected occupied");
    }
    assert_eq!(t.get(bytes("x").iter()), Some(&7));
}

#[test]
fn entry_occupied_vacate_does_not_prune() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("foo"), 1);

    if let Entry::Occupied(o) = t.entry(bytes("foo")) {
        assert_eq!(o.vacate(), 1);
    }
    assert_eq!(t.len(), 0);
    // Path stays, structurally — has_prefix sees no values but the tree is intact
    // by virtue of vacate's contract; we can re-insert without re-creating nodes.
    t.insert(bytes("foo"), 2);
    assert_eq!(t.get(bytes("foo").iter()), Some(&2));
}

#[test]
fn entry_dropping_vacant_leaves_trie_unchanged() {
    let mut t: Trie<u8, i32> = Trie::new();
    let _ = t.entry(bytes("never")); // dropped without inserting
    assert!(t.is_empty());
    assert!(!t.has_prefix(bytes("n").iter()));
}

#[test]
fn borrow_bound_accepts_owned_atoms() {
    // Borrow<A> means owned-atom iterators work directly.
    let mut t: Trie<bool, &'static str> = Trie::new();
    t.insert(vec![true, false, true], "tft");

    // Pass owned bools (Copy), not references.
    let bits: Vec<bool> = vec![true, false, true];
    assert_eq!(t.get(bits.iter().copied()), Some(&"tft"));
    assert!(t.contains_key([true, false, true].into_iter()));
}

#[test]
fn vacate_keeps_intermediate_nodes_for_siblings() {
    let mut t: Trie<u8, i32> = Trie::new();
    t.insert(bytes("foobar"), 1);
    t.insert(bytes("foobaz"), 2);

    // Vacate one — sibling and shared prefix stay
    assert_eq!(t.vacate(bytes("foobar").iter()), Some(1));
    assert_eq!(t.get(bytes("foobaz").iter()), Some(&2));
    assert!(t.has_prefix(bytes("fooba").iter()));

    // Re-insert with no new node creation overhead from the caller's POV
    t.insert(bytes("foobar"), 3);
    assert_eq!(t.get(bytes("foobar").iter()), Some(&3));
}

#[test]
fn many_inserts_consistency() {
    let mut t: Trie<u8, usize> = Trie::new();
    let words = [
        "alpha", "beta", "gamma", "delta", "epsilon", "alphabet", "alpine", "alps", "betray",
        "deltoid",
    ];
    for (i, w) in words.iter().enumerate() {
        t.insert(bytes(w), i);
    }
    assert_eq!(t.len(), words.len());
    for (i, w) in words.iter().enumerate() {
        assert_eq!(t.get(bytes(w).iter()), Some(&i));
    }
    // Replace half
    for (i, w) in words.iter().enumerate().take(5) {
        let prev = t.insert(bytes(w), i + 1000);
        assert_eq!(prev, Some(i));
    }
    assert_eq!(t.len(), words.len());
    // Remove all
    for w in words {
        assert!(t.vacate(bytes(w).iter()).is_some());
    }
    assert!(t.is_empty());
}
