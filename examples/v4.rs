//! IPv4-only IpTrie example with mock GeoIP data.
//!
//! Run with: `cargo run --example v4 --features ip`

use std::net::IpAddr;

use ipnet::IpNet;
use pear_trie::ip::IpTrie;

#[derive(Clone, Debug)]
pub struct IpMetadata {
    pub ip: IpAddr,
    pub subnet: IpNet,
    pub asn_num: String,
    pub asn_name: String,
    pub country: String,
    pub city: String,
}

#[derive(Clone, Debug)]
struct GeoEntry {
    subnet: IpNet,
    asn_num: &'static str,
    asn_name: &'static str,
    country: &'static str,
    city: &'static str,
}

fn mock_v4_data() -> Vec<GeoEntry> {
    vec![
        GeoEntry {
            subnet: "1.0.0.0/24".parse().unwrap(),
            asn_num: "AS13335",
            asn_name: "Cloudflare",
            country: "AU",
            city: "Sydney",
        },
        GeoEntry {
            subnet: "8.8.8.0/24".parse().unwrap(),
            asn_num: "AS15169",
            asn_name: "Google LLC",
            country: "US",
            city: "Mountain View",
        },
        GeoEntry {
            subnet: "8.8.0.0/16".parse().unwrap(),
            asn_num: "AS15169",
            asn_name: "Google LLC",
            country: "US",
            city: "Mountain View",
        },
        GeoEntry {
            subnet: "9.9.9.0/24".parse().unwrap(),
            asn_num: "AS19281",
            asn_name: "Quad9",
            country: "CH",
            city: "Zurich",
        },
        GeoEntry {
            subnet: "104.16.0.0/12".parse().unwrap(),
            asn_num: "AS13335",
            asn_name: "Cloudflare",
            country: "US",
            city: "San Francisco",
        },
        GeoEntry {
            subnet: "192.0.2.0/24".parse().unwrap(),
            asn_num: "AS64512",
            asn_name: "TEST-NET-1",
            country: "ZZ",
            city: "Test",
        },
    ]
}

fn lookup(trie: &IpTrie<GeoEntry>, ip: IpAddr) -> Option<IpMetadata> {
    trie.longest_prefix_match(ip).map(|e| IpMetadata {
        ip,
        subnet: e.subnet,
        asn_num: e.asn_num.to_string(),
        asn_name: e.asn_name.to_string(),
        country: e.country.to_string(),
        city: e.city.to_string(),
    })
}

fn main() {
    let mut trie: IpTrie<GeoEntry> = IpTrie::new();
    for entry in mock_v4_data() {
        trie.insert(entry.subnet, entry.clone());
    }

    let queries: Vec<IpAddr> = ["8.8.8.8", "8.8.4.4", "1.0.0.42", "9.9.9.9", "127.0.0.1"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    for ip in queries {
        match lookup(&trie, ip) {
            Some(md) => println!(
                "{} -> {} ({}, {}) {}/{}",
                md.ip, md.asn_num, md.country, md.city, md.subnet, md.asn_name
            ),
            None => println!("{} -> no match", ip),
        }
    }
}
