#![cfg(feature = "ip")]

use std::net::IpAddr;

use ipnet::IpNet;
use pear_trie::Entry;
use pear_trie::ip::IpTrie;

fn net(s: &str) -> IpNet {
    s.parse().unwrap()
}

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}

#[test]
fn empty_trie_returns_none() {
    let t: IpTrie<u32> = IpTrie::new();
    assert!(t.longest_prefix_match(ip("8.8.8.8")).is_none());
    assert!(t.longest_prefix_match(ip("2001:db8::1")).is_none());
}

#[test]
fn v4_exact_lookup() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("10.0.0.0/8"), "ten");
    t.insert(net("192.168.0.0/16"), "lan");

    assert_eq!(t.longest_prefix_match(ip("10.1.2.3")), Some(&"ten"));
    assert_eq!(t.longest_prefix_match(ip("192.168.42.1")), Some(&"lan"));
    assert_eq!(t.longest_prefix_match(ip("172.16.0.1")), None);
}

#[test]
fn v4_longest_prefix_wins() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("8.8.0.0/16"), "wide");
    t.insert(net("8.8.8.0/24"), "narrow");
    t.insert(net("8.8.8.8/32"), "exact");

    assert_eq!(t.longest_prefix_match(ip("8.8.8.8")), Some(&"exact"));
    assert_eq!(t.longest_prefix_match(ip("8.8.8.9")), Some(&"narrow"));
    assert_eq!(t.longest_prefix_match(ip("8.8.9.1")), Some(&"wide"));
    assert_eq!(t.longest_prefix_match(ip("9.9.9.9")), None);
}

#[test]
fn v4_default_route() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("0.0.0.0/0"), "default");
    t.insert(net("1.0.0.0/8"), "specific");

    assert_eq!(t.longest_prefix_match(ip("1.2.3.4")), Some(&"specific"));
    assert_eq!(t.longest_prefix_match(ip("99.99.99.99")), Some(&"default"));
}

#[test]
fn v6_exact_and_longest() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("2001:db8::/32"), "doc");
    t.insert(net("2001:db8:1::/48"), "doc-sub");

    assert_eq!(t.longest_prefix_match(ip("2001:db8::1")), Some(&"doc"));
    assert_eq!(
        t.longest_prefix_match(ip("2001:db8:1::abcd")),
        Some(&"doc-sub"),
    );
    assert_eq!(t.longest_prefix_match(ip("2001:dead::1")), None);
}

#[test]
fn v4_and_v6_isolated() {
    // A v4 entry must not be reachable through a v6 lookup, even if the
    // bit pattern of the address looks similar.
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("0.0.0.0/0"), "v4-default");

    // No v6 entries exist; v6 lookup must return None.
    assert!(t.longest_prefix_match(ip("::1")).is_none());
    assert!(t.longest_prefix_match(ip("2001:db8::1")).is_none());
    assert_eq!(t.longest_prefix_match(ip("1.2.3.4")), Some(&"v4-default"));

    t.insert(net("::/0"), "v6-default");
    assert_eq!(t.longest_prefix_match(ip("::1")), Some(&"v6-default"));
    assert_eq!(t.longest_prefix_match(ip("1.2.3.4")), Some(&"v4-default"));
}

#[test]
fn insert_replaces_returns_previous() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    assert_eq!(t.insert(net("10.0.0.0/8"), "a"), None);
    assert_eq!(t.insert(net("10.0.0.0/8"), "b"), Some("a"));
    assert_eq!(t.longest_prefix_match(ip("10.1.1.1")), Some(&"b"));
}

#[test]
fn remove_existing_subnet() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("10.0.0.0/8"), "ten");
    t.insert(net("10.1.0.0/16"), "ten-one");

    assert_eq!(t.remove(net("10.1.0.0/16")), Some("ten-one"));
    // Wider entry still wins for addresses that previously matched the narrower one
    assert_eq!(t.longest_prefix_match(ip("10.1.0.5")), Some(&"ten"));

    assert_eq!(t.remove(net("10.0.0.0/8")), Some("ten"));
    assert!(t.longest_prefix_match(ip("10.1.0.5")).is_none());

    // Removing again yields None
    assert!(t.remove(net("10.0.0.0/8")).is_none());
}

#[test]
fn v6_remove() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("2001:db8::/32"), "doc");
    assert_eq!(t.remove(net("2001:db8::/32")), Some("doc"));
    assert!(t.longest_prefix_match(ip("2001:db8::1")).is_none());
}

#[test]
fn many_routes() {
    let routes = [
        ("1.0.0.0/24", "a"),
        ("1.0.1.0/24", "b"),
        ("1.0.0.0/16", "c"),
        ("2.0.0.0/8", "d"),
        ("2606:4700::/32", "cf6"),
        ("2001:4860::/32", "g6"),
    ];
    let mut t: IpTrie<&'static str> = IpTrie::new();
    for (n, v) in routes {
        t.insert(net(n), v);
    }
    assert_eq!(t.longest_prefix_match(ip("1.0.0.5")), Some(&"a"));
    assert_eq!(t.longest_prefix_match(ip("1.0.1.5")), Some(&"b"));
    assert_eq!(t.longest_prefix_match(ip("1.0.5.5")), Some(&"c"));
    assert_eq!(t.longest_prefix_match(ip("2.3.4.5")), Some(&"d"));
    assert_eq!(t.longest_prefix_match(ip("2606:4700::1")), Some(&"cf6"));
    assert_eq!(t.longest_prefix_match(ip("2001:4860::1")), Some(&"g6"));
    assert!(t.longest_prefix_match(ip("3.3.3.3")).is_none());
}

#[test]
fn boundary_prefix_lengths() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("203.0.113.5/32"), "host");
    assert_eq!(t.longest_prefix_match(ip("203.0.113.5")), Some(&"host"));
    assert_eq!(t.longest_prefix_match(ip("203.0.113.6")), None);

    t.insert(net("2001:db8::1/128"), "v6host");
    assert_eq!(t.longest_prefix_match(ip("2001:db8::1")), Some(&"v6host"));
    assert_eq!(t.longest_prefix_match(ip("2001:db8::2")), None);
}

#[test]
fn len_is_empty_clear() {
    let mut t: IpTrie<u32> = IpTrie::new();
    assert!(t.is_empty());
    assert_eq!(t.len(), 0);

    t.insert(net("10.0.0.0/8"), 1);
    t.insert(net("2001:db8::/32"), 2);
    assert_eq!(t.len(), 2);
    assert!(!t.is_empty());

    t.clear();
    assert!(t.is_empty());
    assert!(t.longest_prefix_match(ip("10.0.0.1")).is_none());
}

#[test]
fn vacate_keeps_path_remove_prunes() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("10.0.0.0/8"), "ten");
    t.insert(net("10.1.0.0/16"), "ten-one");

    // vacate clears the slot; the wider entry still routes
    assert_eq!(t.vacate(net("10.1.0.0/16")), Some("ten-one"));
    assert_eq!(t.longest_prefix_match(ip("10.1.0.5")), Some(&"ten"));
    assert_eq!(t.len(), 1);

    // remove cleans up; behavior at the routing layer is unchanged
    assert_eq!(t.remove(net("10.0.0.0/8")), Some("ten"));
    assert!(t.longest_prefix_match(ip("10.0.0.1")).is_none());
    assert!(t.is_empty());
}

#[test]
fn entry_or_insert_inserts_route() {
    let mut t: IpTrie<i32> = IpTrie::new();
    let v = t.entry(net("192.168.0.0/16")).or_insert(42);
    assert_eq!(*v, 42);
    assert_eq!(t.longest_prefix_match(ip("192.168.1.1")), Some(&42));
}

#[test]
fn entry_and_modify_chains() {
    let mut t: IpTrie<i32> = IpTrie::new();

    // First call: vacant path, inserts default
    t.entry(net("10.0.0.0/8")).and_modify(|v| *v += 1).or_insert(0);
    assert_eq!(t.longest_prefix_match(ip("10.0.0.1")), Some(&0));

    // Second call: occupied path, modifies in place
    t.entry(net("10.0.0.0/8")).and_modify(|v| *v += 1).or_insert(0);
    assert_eq!(t.longest_prefix_match(ip("10.0.0.1")), Some(&1));
}

#[test]
fn entry_occupied_vacate_via_entry() {
    let mut t: IpTrie<&'static str> = IpTrie::new();
    t.insert(net("2001:db8::/32"), "doc");

    if let Entry::Occupied(o) = t.entry(net("2001:db8::/32")) {
        assert_eq!(o.vacate(), "doc");
    } else {
        panic!("expected occupied");
    }
    assert_eq!(t.len(), 0);
}

#[test]
fn v4_and_v6_counted_separately_in_len() {
    let mut t: IpTrie<()> = IpTrie::new();
    t.insert(net("10.0.0.0/8"), ());
    t.insert(net("2001:db8::/32"), ());
    t.insert(net("2606:4700::/32"), ());
    assert_eq!(t.len(), 3);
    assert_eq!(t.remove(net("2001:db8::/32")), Some(()));
    assert_eq!(t.len(), 2);
}
