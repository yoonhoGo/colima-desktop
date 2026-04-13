use crate::commands::proxy::ProxyState;
use crate::mdns::config::{self, ContainerMdnsOverride, MdnsConfig};
use crate::mdns::manager::MdnsManager;
use crate::mdns::sync::{self, MdnsSyncResult};
use std::path::PathBuf;
use tauri::{Manager, State};

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(dir.join("mdns-config.json"))
}

#[tauri::command]
pub async fn mdns_get_config(app: tauri::AppHandle) -> Result<MdnsConfig, String> {
    let path = config_path(&app)?;
    Ok(config::load_config(&path).await)
}

#[tauri::command]
pub async fn mdns_set_config(
    app: tauri::AppHandle,
    state: State<'_, MdnsManager>,
    config: MdnsConfig,
) -> Result<(), String> {
    let path = config_path(&app)?;

    let mut manager = state.lock().await;
    if config.enabled {
        manager.enable()?;
    } else {
        manager.disable();
    }

    config::save_config(&path, &config).await?;
    Ok(())
}

#[tauri::command]
pub async fn mdns_set_container_override(
    app: tauri::AppHandle,
    container_name: String,
    override_config: ContainerMdnsOverride,
) -> Result<(), String> {
    let path = config_path(&app)?;
    let mut config = config::load_config(&path).await;
    config
        .container_overrides
        .insert(container_name, override_config);
    config::save_config(&path, &config).await?;
    Ok(())
}

#[tauri::command]
pub async fn mdns_remove_container_override(
    app: tauri::AppHandle,
    container_name: String,
) -> Result<(), String> {
    let path = config_path(&app)?;
    let mut config = config::load_config(&path).await;
    config.container_overrides.remove(&container_name);
    config::save_config(&path, &config).await?;
    Ok(())
}

#[tauri::command]
pub async fn mdns_sync_containers(
    app: tauri::AppHandle,
    state: State<'_, MdnsManager>,
    proxy_state: State<'_, ProxyState>,
) -> Result<MdnsSyncResult, String> {
    let path = config_path(&app)?;
    let config = config::load_config(&path).await;
    let mut manager = state.lock().await;

    // enabled 상태 동기화
    if config.enabled && !manager.is_enabled() {
        manager.enable()?;
    } else if !config.enabled && manager.is_enabled() {
        manager.disable();
    }

    let result = sync::sync_containers(&mut manager, &config).await?;

    // Sync proxy route table with registered mDNS services
    {
        let mut routes = proxy_state.routes.lock().await;
        routes.clear();
        for service in &result.services {
            if service.registered && service.port > 0 {
                let key = format!("{}.local", service.hostname);
                routes.insert(key, service.port);
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn mdns_get_status(
    app: tauri::AppHandle,
    state: State<'_, MdnsManager>,
) -> Result<MdnsStatusResponse, String> {
    let path = config_path(&app)?;
    let config = config::load_config(&path).await;
    let manager = state.lock().await;
    Ok(MdnsStatusResponse {
        enabled: config.enabled,
        daemon_running: manager.is_enabled(),
        registered_count: manager.registered.len(),
        services: manager.list_services(),
    })
}

#[derive(serde::Serialize)]
pub struct MdnsStatusResponse {
    pub enabled: bool,
    pub daemon_running: bool,
    pub registered_count: usize,
    pub services: Vec<crate::mdns::manager::RegisteredService>,
}
