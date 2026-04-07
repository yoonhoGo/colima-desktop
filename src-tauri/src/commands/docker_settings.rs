use crate::cli::types::DockerDaemonSettings;

fn colima_yaml_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(std::path::PathBuf::from(home).join(".colima/default/colima.yaml"))
}

#[tauri::command]
pub async fn get_docker_settings() -> Result<DockerDaemonSettings, String> {
    let path = colima_yaml_path()?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read colima.yaml: {}", e))?;

    let doc: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let docker = doc.get("docker");

    let insecure_registries: Vec<String> = docker
        .and_then(|d| d.get("insecure-registries"))
        .and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_default();

    let registry_mirrors: Vec<String> = docker
        .and_then(|d| d.get("registry-mirrors"))
        .and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(DockerDaemonSettings {
        insecure_registries,
        registry_mirrors,
    })
}

#[tauri::command]
pub async fn save_docker_settings(
    insecure_registries: Vec<String>,
    registry_mirrors: Vec<String>,
) -> Result<(), String> {
    let path = colima_yaml_path()?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read colima.yaml: {}", e))?;

    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let root = doc
        .as_mapping_mut()
        .ok_or("colima.yaml root is not a mapping".to_string())?;

    let docker_key = serde_yaml::Value::String("docker".to_string());

    // Ensure docker key exists as a mapping
    if !root.contains_key(&docker_key) {
        root.insert(
            docker_key.clone(),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
    }

    let docker_mapping = root
        .get_mut(&docker_key)
        .and_then(|v| v.as_mapping_mut())
        .ok_or("docker key is not a mapping".to_string())?;

    let insecure_value = serde_yaml::to_value(&insecure_registries)
        .map_err(|e| format!("Failed to serialize insecure-registries: {}", e))?;
    let mirrors_value = serde_yaml::to_value(&registry_mirrors)
        .map_err(|e| format!("Failed to serialize registry-mirrors: {}", e))?;

    docker_mapping.insert(
        serde_yaml::Value::String("insecure-registries".to_string()),
        insecure_value,
    );
    docker_mapping.insert(
        serde_yaml::Value::String("registry-mirrors".to_string()),
        mirrors_value,
    );

    let output =
        serde_yaml::to_string(&doc).map_err(|e| format!("Failed to serialize YAML: {}", e))?;

    tokio::fs::write(&path, output)
        .await
        .map_err(|e| format!("Failed to write colima.yaml: {}", e))?;

    Ok(())
}
