use crate::cli::types::{DnsHostEntry, NetworkSettings};

fn colima_yaml_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(std::path::PathBuf::from(home).join(".colima/default/colima.yaml"))
}

#[tauri::command]
pub async fn get_network_settings() -> Result<NetworkSettings, String> {
    let path = colima_yaml_path()?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read colima.yaml: {}", e))?;

    let doc: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    // Extract dns (top-level array of strings)
    let dns: Vec<String> = match doc.get("dns") {
        Some(val) => {
            if let Some(arr) = val.as_sequence() {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                vec![]
            }
        }
        None => vec![],
    };

    // Extract dnsHosts (top-level map of hostname -> IP)
    let dns_hosts: Vec<DnsHostEntry> = match doc.get("dnsHosts") {
        Some(val) => {
            if let Some(mapping) = val.as_mapping() {
                mapping
                    .iter()
                    .filter_map(|(k, v)| {
                        let hostname = k.as_str()?.to_string();
                        let ip = v.as_str()?.to_string();
                        Some(DnsHostEntry { hostname, ip })
                    })
                    .collect()
            } else {
                vec![]
            }
        }
        None => vec![],
    };

    // Extract network.address (bool)
    let network_address = doc
        .get("network")
        .and_then(|n| n.get("address"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Extract network.mode or default to "shared"
    let network_mode = doc
        .get("network")
        .and_then(|n| n.get("mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("shared")
        .to_string();

    // Extract network.gatewayAddress or default to ""
    let gateway_address = doc
        .get("network")
        .and_then(|n| n.get("gatewayAddress"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Extract network.interface or default to "en0"
    let network_interface = doc
        .get("network")
        .and_then(|n| n.get("interface"))
        .and_then(|v| v.as_str())
        .unwrap_or("en0")
        .to_string();

    // Extract portForwarder or default to "ssh"
    let port_forwarder = doc
        .get("portForwarder")
        .and_then(|v| v.as_str())
        .unwrap_or("ssh")
        .to_string();

    Ok(NetworkSettings {
        dns,
        dns_hosts,
        network_address,
        network_mode,
        gateway_address,
        network_interface,
        port_forwarder,
    })
}

#[tauri::command]
pub async fn save_network_settings(
    dns: Vec<String>,
    dns_hosts: Vec<DnsHostEntry>,
    network_address: bool,
    network_mode: String,
    gateway_address: String,
    network_interface: String,
    port_forwarder: String,
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

    // Set dns (top-level)
    let dns_value = serde_yaml::Value::Sequence(
        dns.into_iter()
            .map(serde_yaml::Value::String)
            .collect(),
    );
    mapping.insert(
        serde_yaml::Value::String("dns".to_string()),
        dns_value,
    );

    // Set dnsHosts (top-level map)
    let mut dns_hosts_map = serde_yaml::Mapping::new();
    for entry in dns_hosts {
        dns_hosts_map.insert(
            serde_yaml::Value::String(entry.hostname),
            serde_yaml::Value::String(entry.ip),
        );
    }
    mapping.insert(
        serde_yaml::Value::String("dnsHosts".to_string()),
        serde_yaml::Value::Mapping(dns_hosts_map),
    );

    // Set network (nested object) - preserve existing fields
    let network_key = serde_yaml::Value::String("network".to_string());
    let network_val = mapping
        .entry(network_key.clone())
        .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    if let Some(net_map) = network_val.as_mapping_mut() {
        net_map.insert(
            serde_yaml::Value::String("address".to_string()),
            serde_yaml::Value::Bool(network_address),
        );
        net_map.insert(
            serde_yaml::Value::String("mode".to_string()),
            serde_yaml::Value::String(network_mode),
        );
        net_map.insert(
            serde_yaml::Value::String("gatewayAddress".to_string()),
            serde_yaml::Value::String(gateway_address),
        );
        net_map.insert(
            serde_yaml::Value::String("interface".to_string()),
            serde_yaml::Value::String(network_interface),
        );
    }

    // Set portForwarder (top-level)
    mapping.insert(
        serde_yaml::Value::String("portForwarder".to_string()),
        serde_yaml::Value::String(port_forwarder),
    );

    let output =
        serde_yaml::to_string(&doc).map_err(|e| format!("Failed to serialize YAML: {}", e))?;

    tokio::fs::write(&path, output)
        .await
        .map_err(|e| format!("Failed to write colima.yaml: {}", e))?;

    Ok(())
}
