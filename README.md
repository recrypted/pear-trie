# pear-trie

A generic prefix tree (trie) for Rust, where the per-node child storage is
chosen automatically based on the atom type that makes up your keys.

The point of `pear-trie` is that "trie" is really a family of data structures —
a `bool`-keyed bit trie, a `u8`-keyed text trie, and a `String`-keyed dictionary
have very different optimal storage at each node. Rather than picking one
trade-off and forcing it on every atom type, this crate routes the choice
through the [`Atom`] trait: `bool` gets a two-slot array, `u8` gets a sparse
vector tuned for human-readable text, anything `Ord` gets a `BTreeMap`, and so
on. You can override the choice for your own types.

It's small (one core type, a handful of storage backends), generic, and stays
out of your way.

## When to reach for this

- **Prefix lookups over byte strings or words** — autocomplete tables,
  command dispatchers, simple lexers. The `u8` and `String` defaults handle
  these without ceremony.
- **Bit-level keys** — Bloom filter alternatives, decision trees, anything
  where each step in the key is a binary choice. The `bool` atom maps to a
  two-slot array, so each node is exactly the size of two `Option<V>`s.
- **IP routing tables** — see the `ip` feature below. Longest-prefix-match
  for IPv4 and IPv6 falls out of the binary trie naturally.
- **Custom alphabets** — DNA/RNA sequences (4 atoms), hex digits (16),
  opcode sets, state-machine transitions. Implement [`Atom`] for the type
  with whichever storage fits.

## Quick start

```rust
use pear_trie::Trie;

let mut t: Trie<u8, &str> = Trie::new();
t.insert(b"apple".to_vec(), "fruit");
t.insert(b"app".to_vec(),   "prefix");

assert_eq!(t.get(b"apple".iter()), Some(&"fruit"));
assert!(t.has_prefix(b"app".iter()));

// Longest-prefix match: returns the deepest stored key that's a prefix.
let (depth, val) = t.longest_prefix_match(b"applesauce".iter()).unwrap();
assert_eq!(depth, 5);
assert_eq!(val, &"fruit");
```

The `Entry` API is there too, with the usual `or_insert`, `or_insert_with`,
and `and_modify`:

```rust
use pear_trie::Trie;

let mut counts: Trie<u8, u32> = Trie::new();
for word in ["the", "quick", "the", "fox", "quick", "the"] {
    *counts.entry(word.bytes().collect::<Vec<_>>()).or_insert(0) += 1;
}
assert_eq!(counts.get(b"the".iter()), Some(&3));
```

## Removing entries

`pear-trie` distinguishes two flavors of removal, since the right one depends
on what you're doing:

- `Trie::remove(key)` clears the value and prunes any nodes that become empty
  as a result. This is the default — it keeps the trie's memory footprint
  matching the keys actually present.
- `Trie::vacate(key)` clears the value but leaves the path intact. Use this
  when you expect to re-insert nearby keys soon and want to keep the path
  warm. It's a real win for churn within a stable prefix region.

Both return the removed value if there was one.

## Features

| Feature | Default | What it adds |
|---------|---------|--------------|
| `ip`    | off     | `IpTrie<V>`, a longest-prefix-match table for `IpAddr`, backed by two binary tries (one per address family). Pulls in `ipnet`. |

```toml
[dependencies]
pear-trie = { version = "0.1", features = ["ip"] }
```

```rust
# #[cfg(feature = "ip")] {
use pear_trie::ip::IpTrie;

let mut routes: IpTrie<&str> = IpTrie::new();
routes.insert("8.8.8.0/24".parse().unwrap(), "google");
routes.insert("0.0.0.0/0".parse().unwrap(),  "default");

assert_eq!(routes.longest_prefix_match("8.8.8.8".parse().unwrap()), Some(&"google"));
assert_eq!(routes.longest_prefix_match("1.2.3.4".parse().unwrap()), Some(&"default"));
# }
```

The crate is structured so that more wrappers (file paths, URL components,
DNS names, etc.) can be added behind their own feature flags without touching
the core trie.

## Storage backends

You usually don't need to think about these — the `Atom` impl picks one for
you — but they're worth knowing about if you're tuning performance or
implementing `Atom` for your own type:

- `BitStorage` — two-slot array for binary alphabets (`bool`).
- `ArrayStorage<A, V, N>` — fixed-size indexed array for small known
  alphabets (`DenseByte` for dense `u8`, hex digits, etc.).
- `ByteSparseStorage` — linear-scan `Vec<(u8, V)>`, good for sparsely-used
  byte alphabets like ASCII text.
- `SortedVecStorage` — sorted `Vec`, binary-search lookups; the read-mostly
  middle ground.
- `BTreeStorage` — the default for any `Ord` atom; small constant factor and
  cache-friendly.
- `HashStorage` — fallback for `Eq + Hash` atoms that aren't `Ord`.

## Status

This is `0.1`. The public API surface is small and intended to stay stable —
breaking changes will be rare and called out clearly. Documentation lives at
[docs.rs/pear-trie](https://docs.rs/pear-trie).

## License

Licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms or
conditions.

[`Atom`]: https://docs.rs/pear-trie/latest/pear_trie/trait.Atom.html
