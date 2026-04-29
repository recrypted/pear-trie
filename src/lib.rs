//! A generic trie keyed by sequences of *atoms*, with storage strategies tuned
//! per atom type.
//!
//! The core type is [`Trie<A, V>`], a prefix tree where keys are sequences of
//! `A` and each node stores an optional `V`. The [`Atom`] trait picks the
//! per-node child storage automatically — `bool` keys get a two-slot array,
//! `u8` keys get a sparse vector, `Ord` types get a `BTreeMap`, and so on. You
//! can implement [`Atom`] for your own types if none of the defaults fit.
//!
//! # Quick start
//!
//! ```
//! use pear_trie::Trie;
//!
//! let mut t: Trie<u8, &str> = Trie::new();
//! t.insert(b"apple".to_vec(), "fruit");
//! t.insert(b"app".to_vec(),   "prefix");
//!
//! assert_eq!(t.get(b"apple".iter()), Some(&"fruit"));
//! assert!(t.has_prefix(b"app".iter()));
//! ```
//!
//! # Optional features
//!
//! * `ip` — enables [`ip::IpTrie`], a longest-prefix-match table for
//!   [`std::net::IpAddr`] backed by two binary tries.
//!
//! [`ip::IpTrie`]: crate::ip::IpTrie

mod atom;
mod storage;
mod trie;
pub use atom::*;
pub use storage::*;
pub use trie::*;
