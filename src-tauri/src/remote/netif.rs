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

pub(crate) fn is_usable_ipv4(ip: &IpAddr) -> bool {
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

#[cfg(test)]
mod tests {
    use super::is_usable_ipv4;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn is_usable_ipv4_rejects_loopback_and_link_local() {
        assert!(!is_usable_ipv4(&IpAddr::V4(Ipv4Addr::LOCALHOST)));
        assert!(!is_usable_ipv4(&IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));
        assert!(!is_usable_ipv4(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn is_usable_ipv4_accepts_rfc1918() {
        assert!(is_usable_ipv4(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))));
    }
}
