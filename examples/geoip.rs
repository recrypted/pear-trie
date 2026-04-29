//! Mixed IPv4 + IPv6 IpTrie example with mock GeoIP data.
//!
//! Run with: `cargo run --example geoip --features ip`

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

fn mock_data() -> Vec<GeoEntry> {
    vec![
        // IPv4
        GeoEntry {
            subnet: "8.8.8.0/24".parse().unwrap(),
            asn_num: "AS15169",
            asn_name: "Google LLC",
            country: "US",
            city: "Mountain View",
        },
        GeoEntry {
            subnet: "1.1.1.0/24".parse().unwrap(),
            asn_num: "AS13335",
            asn_name: "Cloudflare",
            country: "US",
            city: "San Francisco",
        },
        GeoEntry {
            subnet: "203.0.113.0/24".parse().unwrap(),
            asn_num: "AS64500",
            asn_name: "DocumentationNet",
            country: "ZZ",
            city: "Doc",
        },
        // IPv6
        GeoEntry {
            subnet: "2001:4860::/32".parse().unwrap(),
            asn_num: "AS15169",
            asn_name: "Google LLC",
            country: "US",
            city: "Mountain View",
        },
        GeoEntry {
            subnet: "2606:4700::/32".parse().unwrap(),
            asn_num: "AS13335",
            asn_name: "Cloudflare",
            country: "US",
            city: "San Francisco",
        },
        GeoEntry {
            subnet: "2620:fe::/48".parse().unwrap(),
            asn_num: "AS19281",
            asn_name: "Quad9",
            country: "CH",
            city: "Zurich",
        },
        GeoEntry {
            subnet: "2001:db8::/32".parse().unwrap(),
            asn_num: "AS64501",
            asn_name: "DocumentationNetV6",
            country: "ZZ",
            city: "Doc",
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
    for entry in mock_data() {
        trie.insert(entry.subnet, entry.clone());
    }

    let queries: Vec<IpAddr> = [
        "8.8.8.8",
        "1.1.1.1",
        "203.0.113.42",
        "10.0.0.1",
        "2001:4860:4860::8888",
        "2606:4700:4700::1111",
        "2620:fe::9",
        "2001:db8::1",
        "fe80::1",
    ]
    .iter()
    .map(|s| s.parse().unwrap())
    .collect();

    for ip in queries {
        match lookup(&trie, ip) {
            Some(md) => println!(
                "{} -> {} ({}, {}) subnet={} name={}",
                md.ip, md.asn_num, md.country, md.city, md.subnet, md.asn_name
            ),
            None => println!("{} -> no match", ip),
        }
    }
}
