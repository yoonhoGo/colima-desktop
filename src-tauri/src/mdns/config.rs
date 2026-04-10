use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MdnsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub auto_register: bool,
    #[serde(default = "default_service_type")]
    pub default_service_type: String,
    #[serde(default)]
    pub container_overrides: HashMap<String, ContainerMdnsOverride>,
}

fn default_true() -> bool {
    true
}

fn default_service_type() -> String {
    "_http._tcp.local.".to_string()
}

impl Default for MdnsConfig {
    fn default() -> Self {
        MdnsConfig {
            enabled: false,
            auto_register: true,
            default_service_type: default_service_type(),
            container_overrides: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerMdnsOverride {
    pub enabled: bool,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub service_type: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
}

pub async fn load_config(config_path: &PathBuf) -> MdnsConfig {
    match tokio::fs::read_to_string(config_path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => MdnsConfig::default(),
    }
}

pub async fn save_config(config_path: &PathBuf, config: &MdnsConfig) -> Result<(), String> {
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    tokio::fs::write(config_path, json)
        .await
        .map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MdnsConfig::default();
        assert!(!config.enabled);
        assert!(config.auto_register);
        assert_eq!(config.default_service_type, "_http._tcp.local.");
        assert!(config.container_overrides.is_empty());
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let mut config = MdnsConfig::default();
        config.enabled = true;
        config.container_overrides.insert(
            "my-container".to_string(),
            ContainerMdnsOverride {
                enabled: true,
                hostname: Some("custom.local".to_string()),
                service_type: None,
                port: Some(8080),
            },
        );
        let json = serde_json::to_string(&config).unwrap();
        let parsed: MdnsConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.container_overrides.len(), 1);
        let ovr = parsed.container_overrides.get("my-container").unwrap();
        assert_eq!(ovr.hostname, Some("custom.local".to_string()));
        assert_eq!(ovr.port, Some(8080));
    }

    #[test]
    fn test_config_deserialize_empty_json() {
        let parsed: MdnsConfig = serde_json::from_str("{}").unwrap();
        assert!(!parsed.enabled);
        assert!(parsed.auto_register);
    }
}
