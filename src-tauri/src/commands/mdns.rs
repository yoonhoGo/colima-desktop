use crate::cli::types::{MdnsProperty, MdnsRegistration, MdnsServiceEntry, MdnsState};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

pub struct MdnsManager {
    daemon: ServiceDaemon,
    registered: Vec<MdnsRegistration>,
    enabled: bool,
}

impl MdnsManager {
    pub fn new() -> Result<Self, String> {
        let daemon =
            ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
        Ok(Self {
            daemon,
            registered: Vec::new(),
            enabled: false,
        })
    }
}

pub type MdnsManagerState = Arc<Mutex<Option<MdnsManager>>>;

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
    let guard = state.lock().await;
    match guard.as_ref() {
        Some(manager) => Ok(MdnsState {
            enabled: true,
            registered_services: manager.registered.clone(),
            discovered_services: vec![],
        }),
        None => Ok(MdnsState {
            enabled: false,
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

    // Ensure service_type ends with ".local."
    let stype = if service_type.ends_with(".local.") {
        service_type.clone()
    } else if service_type.ends_with('.') {
        format!("{}local.", service_type)
    } else {
        format!("{}.local.", service_type)
    };

    let hostname = format!("{}.local.", instance_name);

    let mut prop_map: HashMap<String, String> = HashMap::new();
    for p in &properties {
        prop_map.insert(p.key.clone(), p.value.clone());
    }

    let props: Vec<(&str, &str)> = prop_map
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let service = ServiceInfo::new(&stype, &instance_name, &hostname, (), port, &props[..])
        .map_err(|e| format!("Failed to create service info: {}", e))?;

    manager
        .daemon
        .register(service)
        .map_err(|e| format!("Failed to register service: {}", e))?;

    manager.registered.push(MdnsRegistration {
        instance_name,
        service_type: stype,
        port,
        properties: properties,
    });

    Ok(())
}

#[tauri::command]
pub async fn mdns_unregister_service(
    state: State<'_, MdnsManagerState>,
    instance_name: String,
    service_type: String,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    let manager = guard.as_mut().ok_or("mDNS is not enabled")?;

    let stype = if service_type.ends_with(".local.") {
        service_type.clone()
    } else if service_type.ends_with('.') {
        format!("{}local.", service_type)
    } else {
        format!("{}.local.", service_type)
    };

    let fullname = format!("{}.{}", instance_name, stype);
    let _ = manager.daemon.unregister(&fullname);

    manager
        .registered
        .retain(|r| !(r.instance_name == instance_name && r.service_type == stype));

    Ok(())
}

#[tauri::command]
pub async fn mdns_browse(
    state: State<'_, MdnsManagerState>,
    service_type: String,
) -> Result<Vec<MdnsServiceEntry>, String> {
    let guard = state.lock().await;
    let manager = guard.as_ref().ok_or("mDNS is not enabled")?;

    let stype = if service_type.ends_with(".local.") {
        service_type.clone()
    } else if service_type.ends_with('.') {
        format!("{}local.", service_type)
    } else {
        format!("{}.local.", service_type)
    };

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

    mdns_register_service(state, container_name, stype, port, properties).await
}
