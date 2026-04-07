use crate::cli::executor::CliExecutor;
use crate::cli::types::{DockerNetworkEntry, Network};

const DOCKER: &str = "/opt/homebrew/bin/docker";

#[tauri::command]
pub async fn list_networks() -> Result<Vec<Network>, String> {
    let entries: Vec<DockerNetworkEntry> =
        CliExecutor::run_json_lines(DOCKER, &["network", "ls", "--format", "json"]).await?;
    Ok(entries.into_iter().map(Network::from).collect())
}

#[tauri::command]
pub async fn create_network(name: String, driver: Option<String>) -> Result<String, String> {
    let mut args = vec!["network", "create"];
    let driver_val;
    if let Some(ref d) = driver {
        if !d.is_empty() && d != "bridge" {
            driver_val = d.clone();
            args.push("--driver");
            args.push(&driver_val);
        }
    }
    args.push(&name);
    CliExecutor::run(DOCKER, &args).await
}

#[tauri::command]
pub async fn remove_network(id: String) -> Result<(), String> {
    CliExecutor::run(DOCKER, &["network", "rm", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn prune_networks() -> Result<String, String> {
    CliExecutor::run(DOCKER, &["network", "prune", "-f"]).await
}
