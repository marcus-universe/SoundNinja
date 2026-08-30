use local_ip_address::{list_afinet_netifas, local_ip};
use serde::Serialize;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalIpInfo {
    pub name: String,
    pub ip: String,
    pub primary: bool,
}

fn is_usable_ipv4(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            !v4.is_loopback() && !v4.is_link_local() && !v4.is_unspecified() && !v4.is_multicast()
        }
        IpAddr::V6(_) => false,
    }
}

pub fn list_local_ips() -> Vec<LocalIpInfo> {
    let primary = local_ip().ok().filter(|ip| is_usable_ipv4(ip));
    let ifaces = list_afinet_netifas().unwrap_or_default();
    let mut out = Vec::new();
    for (name, ip) in ifaces {
        if !is_usable_ipv4(&ip) {
            continue;
        }
        let is_primary = primary.map(|p| p == ip).unwrap_or(false);
        out.push(LocalIpInfo {
            name,
            ip: ip.to_string(),
            primary: is_primary,
        });
    }
    if out.iter().all(|e| !e.primary) {
        if let Some(first) = out.first_mut() {
            first.primary = true;
        }
    }
    out.sort_by(|a, b| b.primary.cmp(&a.primary).then(a.ip.cmp(&b.ip)));
    out
}
