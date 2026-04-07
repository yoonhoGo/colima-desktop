use crate::cli::executor::CliExecutor;
use crate::cli::types::{DockerVolumeEntry, Volume};

const DOCKER: &str = "/opt/homebrew/bin/docker";

#[tauri::command]
pub async fn list_volumes() -> Result<Vec<Volume>, String> {
    let entries: Vec<DockerVolumeEntry> =
        CliExecutor::run_json_lines(DOCKER, &["volume", "ls", "--format", "json"]).await?;
    Ok(entries.into_iter().map(Volume::from).collect())
}

#[tauri::command]
pub async fn create_volume(name: String, driver: Option<String>) -> Result<String, String> {
    let mut args = vec!["volume", "create"];
    let driver_val;
    if let Some(ref d) = driver {
        if !d.is_empty() && d != "local" {
            driver_val = d.clone();
            args.push("--driver");
            args.push(&driver_val);
        }
    }
    args.push(&name);
    CliExecutor::run(DOCKER, &args).await
}

#[tauri::command]
pub async fn remove_volume(name: String) -> Result<(), String> {
    CliExecutor::run(DOCKER, &["volume", "rm", &name]).await?;
    Ok(())
}

#[tauri::command]
pub async fn prune_volumes() -> Result<String, String> {
    CliExecutor::run(DOCKER, &["volume", "prune", "-f"]).await
}
