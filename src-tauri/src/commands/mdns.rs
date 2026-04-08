use crate::cli::executor::CliExecutor;
use crate::cli::types::{
    ContainerMdnsConfig, DockerPsEntry, MdnsPersistentConfig, MdnsProperty, MdnsRegistration,
    MdnsServiceEntry, MdnsState,
};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

const DOCKER: &str = "/opt/homebrew/bin/docker";

/// Get the host machine's local (LAN) IPv4 address.
/// This is the IP that other devices on the network can reach.
fn get_host_ip() -> Ipv4Addr {
    local_ip_address::local_ip()
        .ok()
        .and_then(|ip| match ip {
            std::net::IpAddr::V4(v4) => Some(v4),
            _ => None,
        })
        .unwrap_or(Ipv4Addr::LOCALHOST)
}

/// Get the machine's hostname for mDNS, falling back to "colima-host".
fn get_mdns_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "colima-host".to_string())
}

pub struct MdnsManager {
    daemon: ServiceDaemon,
    registered: Vec<MdnsRegistration>,
}

impl MdnsManager {
    pub fn new() -> Result<Self, String> {
        let daemon =
            ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
        Ok(Self {
            daemon,
            registered: Vec::new(),
        })
    }

    fn register_service_inner(
        &mut self,
        instance_name: &str,
        service_type: &str,
        port: u16,
        properties: &[MdnsProperty],
    ) -> Result<(), String> {
        let stype = normalize_service_type(service_type);

        // Use the machine's actual hostname so the A record resolves correctly
        let hostname = format!("{}.local.", get_mdns_hostname());

        // Get the host's LAN IP so mDNS responses include an A record
        let host_ip = get_host_ip();

        let mut prop_map: HashMap<String, String> = HashMap::new();
        for p in properties {
            prop_map.insert(p.key.clone(), p.value.clone());
        }

        let props: Vec<(&str, &str)> = prop_map
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let service =
            ServiceInfo::new(&stype, instance_name, &hostname, host_ip, port, &props[..])
                .map_err(|e| format!("Failed to create service info: {}", e))?;

        self.daemon
            .register(service)
            .map_err(|e| format!("Failed to register service: {}", e))?;

        // Avoid duplicates
        if !self
            .registered
            .iter()
            .any(|r| r.instance_name == instance_name && r.service_type == stype)
        {
            self.registered.push(MdnsRegistration {
                instance_name: instance_name.to_string(),
                service_type: stype,
                port,
                properties: properties.to_vec(),
            });
        }

        Ok(())
    }

    fn unregister_service_inner(
        &mut self,
        instance_name: &str,
        service_type: &str,
    ) -> Result<(), String> {
        let stype = normalize_service_type(service_type);
        let fullname = format!("{}.{}", instance_name, stype);
        let _ = self.daemon.unregister(&fullname);
        self.registered
            .retain(|r| !(r.instance_name == instance_name && r.service_type == stype));
        Ok(())
    }
}

fn normalize_service_type(service_type: &str) -> String {
    if service_type.ends_with(".local.") {
        service_type.to_string()
    } else if service_type.ends_with('.') {
        format!("{}local.", service_type)
    } else {
        format!("{}.local.", service_type)
    }
}

pub type MdnsManagerState = Arc<Mutex<Option<MdnsManager>>>;

fn config_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(std::path::PathBuf::from(home).join(".colima-desktop/mdns-config.json"))
}

async fn load_config() -> MdnsPersistentConfig {
    let path = match config_path() {
        Ok(p) => p,
        Err(_) => return MdnsPersistentConfig::default(),
    };
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => MdnsPersistentConfig::default(),
    }
}

async fn save_config(config: &MdnsPersistentConfig) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize: {}", e))?;
    tokio::fs::write(&path, json)
        .await
        .map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

pub fn create_mdns_state() -> MdnsManagerState {
    Arc::new(Mutex::new(None))
}

#[tauri::command]
pub async fn mdns_enable(state: State<'_, MdnsManagerState>) -> Result<(), String> {
    let mut guard = state.lock().await;
    if guard.is_some() {
        return Ok(());
    }
    let manager = MdnsManager::new()?;
    *guard = Some(manager);
    Ok(())
}

#[tauri::command]
pub async fn mdns_disable(state: State<'_, MdnsManagerState>) -> Result<(), String> {
    let mut guard = state.lock().await;
    if let Some(manager) = guard.take() {
        let _ = manager.daemon.shutdown();
    }
    Ok(())
}

#[tauri::command]
pub async fn mdns_get_state(state: State<'_, MdnsManagerState>) -> Result<MdnsState, String> {
    let config = load_config().await;
    let guard = state.lock().await;
    match guard.as_ref() {
        Some(manager) => Ok(MdnsState {
            enabled: true,
            auto_register: config.auto_register,
            registered_services: manager.registered.clone(),
            discovered_services: vec![],
        }),
        None => Ok(MdnsState {
            enabled: false,
            auto_register: config.auto_register,
            registered_services: vec![],
            discovered_services: vec![],
        }),
    }
}

#[tauri::command]
pub async fn mdns_register_service(
    state: State<'_, MdnsManagerState>,
    instance_name: String,
    service_type: String,
    port: u16,
    properties: Vec<MdnsProperty>,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    let manager = guard.as_mut().ok_or("mDNS is not enabled")?;
    manager.register_service_inner(&instance_name, &service_type, port, &properties)
}

#[tauri::command]
pub async fn mdns_unregister_service(
    state: State<'_, MdnsManagerState>,
    instance_name: String,
    service_type: String,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    let manager = guard.as_mut().ok_or("mDNS is not enabled")?;
    manager.unregister_service_inner(&instance_name, &service_type)
}

#[tauri::command]
pub async fn mdns_browse(
    state: State<'_, MdnsManagerState>,
    service_type: String,
) -> Result<Vec<MdnsServiceEntry>, String> {
    let guard = state.lock().await;
    let manager = guard.as_ref().ok_or("mDNS is not enabled")?;

    let stype = normalize_service_type(&service_type);

    let receiver = manager
        .daemon
        .browse(&stype)
        .map_err(|e| format!("Failed to browse: {}", e))?;

    let mut services = Vec::new();
    let timeout = std::time::Duration::from_secs(3);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match receiver.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                let properties: Vec<MdnsProperty> = info
                    .get_properties()
                    .iter()
                    .map(|prop| MdnsProperty {
                        key: prop.key().to_string(),
                        value: prop.val_str().to_string(),
                    })
                    .collect();

                services.push(MdnsServiceEntry {
                    instance_name: info.get_fullname().to_string(),
                    service_type: info.get_type().to_string(),
                    hostname: info.get_hostname().to_string(),
                    port: info.get_port(),
                    addresses: info.get_addresses().iter().map(|a| a.to_string()).collect(),
                    properties,
                });
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }

    Ok(services)
}

#[tauri::command]
pub async fn mdns_register_container(
    state: State<'_, MdnsManagerState>,
    container_name: String,
    port: u16,
    service_type: Option<String>,
) -> Result<(), String> {
    let stype = service_type.unwrap_or_else(|| "_http._tcp".to_string());
    let properties = vec![
        MdnsProperty {
            key: "source".to_string(),
            value: "colima-desktop".to_string(),
        },
        MdnsProperty {
            key: "container".to_string(),
            value: container_name.clone(),
        },
    ];

    let mut guard = state.lock().await;
    let manager = guard.as_mut().ok_or("mDNS is not enabled")?;
    manager.register_service_inner(&container_name, &stype, port, &properties)
}

// --- Per-container mDNS config ---

#[tauri::command]
pub async fn mdns_get_container_configs() -> Result<Vec<ContainerMdnsConfig>, String> {
    let config = load_config().await;
    Ok(config.containers)
}

#[tauri::command]
pub async fn mdns_set_container_config(
    state: State<'_, MdnsManagerState>,
    container_id: String,
    container_name: String,
    enabled: bool,
    service_type: String,
    port: u16,
) -> Result<(), String> {
    let mut config = load_config().await;

    // Update or insert
    if let Some(existing) = config
        .containers
        .iter_mut()
        .find(|c| c.container_id == container_id)
    {
        existing.enabled = enabled;
        existing.service_type = service_type.clone();
        existing.port = port;
        existing.container_name = container_name.clone();
    } else {
        config.containers.push(ContainerMdnsConfig {
            container_id: container_id.clone(),
            container_name: container_name.clone(),
            enabled,
            service_type: service_type.clone(),
            port,
        });
    }

    save_config(&config).await?;

    // Apply immediately if mDNS is running
    let mut guard = state.lock().await;
    if let Some(manager) = guard.as_mut() {
        if enabled && port > 0 {
            let props = vec![
                MdnsProperty {
                    key: "source".to_string(),
                    value: "colima-desktop".to_string(),
                },
                MdnsProperty {
                    key: "container".to_string(),
                    value: container_name.clone(),
                },
            ];
            let _ = manager.register_service_inner(&container_name, &service_type, port, &props);
        } else {
            let _ = manager.unregister_service_inner(&container_name, &service_type);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn mdns_remove_container_config(
    state: State<'_, MdnsManagerState>,
    container_id: String,
) -> Result<(), String> {
    let mut config = load_config().await;

    // Unregister from mDNS daemon if running
    let removed: Vec<_> = config
        .containers
        .iter()
        .filter(|c| c.container_id == container_id)
        .cloned()
        .collect();

    let mut guard = state.lock().await;
    if let Some(manager) = guard.as_mut() {
        for c in &removed {
            let _ = manager.unregister_service_inner(&c.container_name, &c.service_type);
        }
    }

    config.containers.retain(|c| c.container_id != container_id);
    save_config(&config).await?;
    Ok(())
}

// --- Auto-register ---

#[tauri::command]
pub async fn mdns_set_auto_register(auto_register: bool) -> Result<(), String> {
    let mut config = load_config().await;
    config.auto_register = auto_register;
    save_config(&config).await
}

/// Sync mDNS registrations with currently running containers.
/// Called periodically from the frontend alongside container list polling.
#[tauri::command]
pub async fn mdns_sync_containers(state: State<'_, MdnsManagerState>) -> Result<(), String> {
    let mut guard = state.lock().await;
    let manager = match guard.as_mut() {
        Some(m) => m,
        None => return Ok(()), // mDNS not enabled, nothing to do
    };

    let config = load_config().await;

    // Get running containers
    let entries: Vec<DockerPsEntry> =
        match CliExecutor::run_json_lines(DOCKER, &["ps", "--format", "json"]).await {
            Ok(e) => e,
            Err(_) => return Ok(()), // Docker not available, skip
        };

    let running_names: HashMap<String, &DockerPsEntry> =
        entries.iter().map(|e| (e.names.clone(), e)).collect();

    // Register containers that are configured + running
    for cc in &config.containers {
        if !cc.enabled || cc.port == 0 {
            continue;
        }
        if running_names.contains_key(&cc.container_name) {
            // Ensure registered
            let already = manager
                .registered
                .iter()
                .any(|r| r.instance_name == cc.container_name);
            if !already {
                let props = vec![
                    MdnsProperty {
                        key: "source".to_string(),
                        value: "colima-desktop".to_string(),
                    },
                    MdnsProperty {
                        key: "container".to_string(),
                        value: cc.container_name.clone(),
                    },
                ];
                let _ = manager.register_service_inner(
                    &cc.container_name,
                    &cc.service_type,
                    cc.port,
                    &props,
                );
            }
        } else {
            // Container not running -> unregister
            let _ = manager.unregister_service_inner(&cc.container_name, &cc.service_type);
        }
    }

    // Auto-register: for running containers with exposed ports that have no config yet
    if config.auto_register {
        for entry in &entries {
            let has_config = config
                .containers
                .iter()
                .any(|c| c.container_name == entry.names);
            if has_config {
                continue;
            }

            // Parse first host port from ports string (e.g. "0.0.0.0:8080->80/tcp")
            if let Some(port) = parse_first_host_port(&entry.ports) {
                let already = manager
                    .registered
                    .iter()
                    .any(|r| r.instance_name == entry.names);
                if !already {
                    let props = vec![
                        MdnsProperty {
                            key: "source".to_string(),
                            value: "colima-desktop".to_string(),
                        },
                        MdnsProperty {
                            key: "container".to_string(),
                            value: entry.names.clone(),
                        },
                        MdnsProperty {
                            key: "auto".to_string(),
                            value: "true".to_string(),
                        },
                    ];
                    let _ =
                        manager.register_service_inner(&entry.names, "_http._tcp", port, &props);
                }
            }
        }

        // Unregister auto-registered services for containers that are no longer running
        let auto_registered: Vec<String> = manager
            .registered
            .iter()
            .filter(|r| {
                r.properties
                    .iter()
                    .any(|p| p.key == "auto" && p.value == "true")
            })
            .map(|r| r.instance_name.clone())
            .collect();

        for name in auto_registered {
            if !running_names.contains_key(&name) {
                let _ = manager.unregister_service_inner(&name, "_http._tcp");
            }
        }
    }

    Ok(())
}

/// Parse the first host port from a Docker ports string like "0.0.0.0:8080->80/tcp"
fn parse_first_host_port(ports: &str) -> Option<u16> {
    // Format: "0.0.0.0:8080->80/tcp, :::8080->80/tcp"
    let segment = ports.split(',').next()?;
    let arrow = segment.find("->")?;
    let before_arrow = &segment[..arrow];
    // Extract port after last ':'
    let colon_pos = before_arrow.rfind(':')?;
    let port_str = &before_arrow[colon_pos + 1..];
    port_str.trim().parse::<u16>().ok()
}
