//! IP-prefix routing table built on top of the generic [`Trie`].
//!
//! Available behind the `ip` feature. See [`IpTrie`].

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnet::{IpNet, Ipv4Net, Ipv6Net};

use crate::{Entry, Trie};

/// Longest-prefix-match table mapping IP networks to values of type `V`.
///
/// Internally keeps two binary tries — one for IPv4 and one for IPv6 — with
/// keys that are the network's prefix bits (most-significant first). Lookups
/// take an [`IpAddr`] and return the value associated with the most specific
/// matching network.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "ip")] {
/// use pear_trie::ip::IpTrie;
///
/// let mut t: IpTrie<&str> = IpTrie::new();
/// t.insert("8.8.8.0/24".parse().unwrap(), "google");
/// t.insert("0.0.0.0/0".parse().unwrap(),  "default");
///
/// assert_eq!(t.longest_prefix_match("8.8.8.8".parse().unwrap()), Some(&"google"));
/// assert_eq!(t.longest_prefix_match("1.2.3.4".parse().unwrap()), Some(&"default"));
/// # }
/// ```
pub struct IpTrie<V> {
    v4: Trie<bool, V>,
    v6: Trie<bool, V>,
}

impl<V> Default for IpTrie<V> {
    fn default() -> Self {
        IpTrie::new()
    }
}

impl<V> IpTrie<V> {
    /// Construct an empty `IpTrie`.
    pub fn new() -> Self {
        Self {
            v4: Trie::new(),
            v6: Trie::new(),
        }
    }

    /// Associate `value` with the network `net`. Returns the previous value
    /// if `net` was already present.
    pub fn insert(&mut self, net: IpNet, value: V) -> Option<V> {
        match net {
            IpNet::V4(n) => self.v4.insert(ipv4_net_bits(&n), value),
            IpNet::V6(n) => self.v6.insert(ipv6_net_bits(&n), value),
        }
    }

    /// Find the value attached to the most specific stored network that
    /// contains `ip`.
    ///
    /// IPv4 and IPv6 are looked up in their own tries — a v6 address never
    /// matches a v4 prefix or vice versa.
    pub fn longest_prefix_match(&self, ip: IpAddr) -> Option<&V> {
        match ip {
            IpAddr::V4(a) => self
                .v4
                .longest_prefix_match(ipv4_addr_bits(a))
                .map(|(_, v)| v),
            IpAddr::V6(a) => self
                .v6
                .longest_prefix_match(ipv6_addr_bits(a))
                .map(|(_, v)| v),
        }
    }

    /// Remove the entry for `net` and prune the now-empty branch. Returns
    /// the removed value, if any.
    pub fn remove(&mut self, net: IpNet) -> Option<V> {
        match net {
            IpNet::V4(n) => self.v4.remove(ipv4_net_bits(&n)),
            IpNet::V6(n) => self.v6.remove(ipv6_net_bits(&n)),
        }
    }

    /// Clear the entry for `net` without pruning the path. Returns the
    /// removed value, if any. Useful when you expect to re-insert routes in
    /// the same region soon — see [`Trie::vacate`] for the reasoning.
    pub fn vacate(&mut self, net: IpNet) -> Option<V> {
        match net {
            IpNet::V4(n) => self.v4.vacate(ipv4_net_bits(&n)),
            IpNet::V6(n) => self.v6.vacate(ipv6_net_bits(&n)),
        }
    }

    /// Get the [`Entry`] for `net`. The entry's atom type is `bool` because
    /// the underlying tries are bit-keyed.
    pub fn entry(&mut self, net: IpNet) -> Entry<'_, bool, V> {
        match net {
            IpNet::V4(n) => self.v4.entry(ipv4_net_bits(&n)),
            IpNet::V6(n) => self.v6.entry(ipv6_net_bits(&n)),
        }
    }

    /// Total number of stored networks across IPv4 and IPv6.
    pub fn len(&self) -> usize {
        self.v4.len() + self.v6.len()
    }

    /// Whether the table holds no networks.
    pub fn is_empty(&self) -> bool {
        self.v4.is_empty() && self.v6.is_empty()
    }

    /// Drop every stored entry on both tries.
    pub fn clear(&mut self) {
        self.v4.clear();
        self.v6.clear();
    }
}

fn bits_from_octets<const N: usize>(
    octets: [u8; N],
    prefix_len: usize,
) -> impl Iterator<Item = bool> {
    debug_assert!(prefix_len <= N * 8);
    (0..prefix_len).map(move |i| {
        let byte = octets[i / 8];
        let bit = 7 - (i % 8);
        (byte >> bit) & 1 != 0
    })
}

fn ipv4_net_bits(net: &Ipv4Net) -> impl Iterator<Item = bool> {
    bits_from_octets(net.network().octets(), net.prefix_len() as usize)
}

fn ipv6_net_bits(net: &Ipv6Net) -> impl Iterator<Item = bool> {
    bits_from_octets(net.network().octets(), net.prefix_len() as usize)
}

fn ipv4_addr_bits(addr: Ipv4Addr) -> impl Iterator<Item = bool> {
    bits_from_octets(addr.octets(), 32)
}

fn ipv6_addr_bits(addr: Ipv6Addr) -> impl Iterator<Item = bool> {
    bits_from_octets(addr.octets(), 128)
}
