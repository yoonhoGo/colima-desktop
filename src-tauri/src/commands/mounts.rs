use crate::cli::types::{MountEntry, MountSettings};

fn colima_yaml_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(std::path::PathBuf::from(home).join(".colima/default/colima.yaml"))
}

#[tauri::command]
pub async fn get_mount_settings() -> Result<MountSettings, String> {
    let path = colima_yaml_path()?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read colima.yaml: {}", e))?;

    let doc: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let mounts: Vec<MountEntry> = match doc.get("mounts") {
        Some(val) => {
            serde_yaml::from_value(val.clone()).map_err(|e| format!("Failed to parse mounts: {}", e))?
        }
        None => vec![],
    };

    let mount_type = doc
        .get("mountType")
        .and_then(|v| v.as_str())
        .unwrap_or("sshfs")
        .to_string();

    let mount_inotify = doc
        .get("mountInotify")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(MountSettings {
        mounts,
        mount_type,
        mount_inotify,
    })
}

#[tauri::command]
pub async fn save_mount_settings(
    mounts: Vec<MountEntry>,
    mount_type: String,
    mount_inotify: bool,
) -> Result<(), String> {
    let path = colima_yaml_path()?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read colima.yaml: {}", e))?;

    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let mapping = doc
        .as_mapping_mut()
        .ok_or("colima.yaml root is not a mapping".to_string())?;

    let mounts_value =
        serde_yaml::to_value(&mounts).map_err(|e| format!("Failed to serialize mounts: {}", e))?;
    mapping.insert(
        serde_yaml::Value::String("mounts".to_string()),
        mounts_value,
    );
    mapping.insert(
        serde_yaml::Value::String("mountType".to_string()),
        serde_yaml::Value::String(mount_type),
    );
    mapping.insert(
        serde_yaml::Value::String("mountInotify".to_string()),
        serde_yaml::Value::Bool(mount_inotify),
    );

    let output =
        serde_yaml::to_string(&doc).map_err(|e| format!("Failed to serialize YAML: {}", e))?;

    tokio::fs::write(&path, output)
        .await
        .map_err(|e| format!("Failed to write colima.yaml: {}", e))?;

    Ok(())
}
