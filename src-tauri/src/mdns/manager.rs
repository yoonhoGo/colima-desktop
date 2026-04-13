use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::Serialize;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct RegisteredService {
    pub container_name: String,
    pub hostname: String,
    pub service_type: String,
    pub port: u16,
    pub fullname: String,
    pub auto_registered: bool,
}

pub struct MdnsManagerInner {
    daemon: Option<ServiceDaemon>,
    pub registered: HashMap<String, RegisteredService>,
}

pub type MdnsManager = Arc<Mutex<MdnsManagerInner>>;

pub fn create_mdns_manager() -> MdnsManager {
    Arc::new(Mutex::new(MdnsManagerInner {
        daemon: None,
        registered: HashMap::new(),
    }))
}

impl MdnsManagerInner {
    pub fn enable(&mut self) -> Result<(), String> {
        if self.daemon.is_some() {
            return Ok(());
        }
        let daemon = ServiceDaemon::new()
            .map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
        self.daemon = Some(daemon);
        Ok(())
    }

    pub fn disable(&mut self) {
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.shutdown();
        }
        self.registered.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.daemon.is_some()
    }

    pub fn register(
        &mut self,
        container_name: &str,
        hostname: &str,
        service_type: &str,
        port: u16,
        auto_registered: bool,
        ip: IpAddr,
    ) -> Result<(), String> {
        let daemon = self.daemon.as_ref().ok_or("mDNS daemon not running")?;

        let stype = normalize_service_type(service_type);

        // hostname을 FQDN으로 변환 (e.g., "kafka-ui" → "kafka-ui.local.")
        // 이렇게 해야 kafka-ui.local → IP A 레코드가 생성됨
        let fqdn_host = format!("{}.local.", hostname);

        let fullname = format!("{}.{}", hostname, stype);

        let service = ServiceInfo::new(
            &stype,
            hostname,
            &fqdn_host,
            ip,
            port,
            [("source", "colima-desktop")].as_ref(),
        )
        .map_err(|e| format!("Failed to create ServiceInfo: {}", e))?;

        daemon
            .register(service)
            .map_err(|e| format!("Failed to register service: {}", e))?;

        self.registered.insert(
            container_name.to_string(),
            RegisteredService {
                container_name: container_name.to_string(),
                hostname: hostname.to_string(),
                service_type: stype.clone(),
                port,
                fullname,
                auto_registered,
            },
        );

        Ok(())
    }

    pub fn unregister(&mut self, container_name: &str) -> Result<(), String> {
        if let Some(service) = self.registered.remove(container_name) {
            if let Some(daemon) = &self.daemon {
                let _ = daemon.unregister(&service.fullname);
            }
        }
        Ok(())
    }

    pub fn list_services(&self) -> Vec<RegisteredService> {
        self.registered.values().cloned().collect()
    }
}

impl Drop for MdnsManagerInner {
    fn drop(&mut self) {
        self.disable();
    }
}

/// Resolve the IP address to use for mDNS registration based on ip_mode.
/// - "auto": LAN IP → 127.0.0.1 fallback
/// - "vm": Colima VM IP → LAN IP → 127.0.0.1 fallback
/// - "localhost": always 127.0.0.1
/// - anything else: parse as custom IP
pub async fn resolve_mdns_ip(ip_mode: &str) -> IpAddr {
    match ip_mode {
        "vm" => {
            if let Some(ip) = get_colima_vm_ip().await {
                return ip;
            }
            // fallback to auto
            local_ip_address::local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
        }
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        "auto" | "" => {
            local_ip_address::local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
        }
        custom => custom
            .parse::<IpAddr>()
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
    }
}

/// Get the Colima VM's first routable IP address via `colima ssh -- hostname -I`.
async fn get_colima_vm_ip() -> Option<IpAddr> {
    let output = tokio::process::Command::new("colima")
        .args(["ssh", "--", "hostname", "-I"])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // hostname -I returns space-separated IPs; take the first non-Docker one
    stdout
        .split_whitespace()
        .filter_map(|s| s.parse::<IpAddr>().ok())
        .find(|ip| {
            // Skip Docker bridge IPs (172.17.x.x, 172.18.x.x, etc.)
            if let IpAddr::V4(v4) = ip {
                let octets = v4.octets();
                !(octets[0] == 172 && (16..=31).contains(&octets[1]))
            } else {
                true
            }
        })
}

pub fn normalize_service_type(service_type: &str) -> String {
    if service_type.ends_with(".local.") {
        service_type.to_string()
    } else if service_type.ends_with(".local") {
        format!("{}.", service_type)
    } else if service_type.ends_with('.') {
        format!("{}local.", service_type)
    } else {
        format!("{}.local.", service_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_service_type() {
        assert_eq!(
            normalize_service_type("_http._tcp"),
            "_http._tcp.local."
        );
        assert_eq!(
            normalize_service_type("_http._tcp."),
            "_http._tcp.local."
        );
        assert_eq!(
            normalize_service_type("_http._tcp.local."),
            "_http._tcp.local."
        );
        assert_eq!(
            normalize_service_type("_http._tcp.local"),
            "_http._tcp.local."
        );
    }
}
