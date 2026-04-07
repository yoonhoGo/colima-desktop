use crate::cli::executor::CliExecutor;
use crate::cli::types::{Container, DockerPsEntry};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

const DOCKER: &str = "/opt/homebrew/bin/docker";

#[tauri::command]
pub async fn list_containers() -> Result<Vec<Container>, String> {
    let entries: Vec<DockerPsEntry> =
        CliExecutor::run_json_lines(DOCKER, &["ps", "-a", "--format", "json"]).await?;
    Ok(entries.into_iter().map(Container::from).collect())
}

#[tauri::command]
pub async fn container_start(id: String) -> Result<(), String> {
    CliExecutor::run(DOCKER, &["start", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn container_stop(id: String) -> Result<(), String> {
    CliExecutor::run(DOCKER, &["stop", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn container_restart(id: String) -> Result<(), String> {
    CliExecutor::run(DOCKER, &["restart", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn container_remove(id: String) -> Result<(), String> {
    CliExecutor::run(DOCKER, &["rm", "-f", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn prune_containers() -> Result<String, String> {
    CliExecutor::run(DOCKER, &["container", "prune", "-f"]).await
}

#[tauri::command]
pub async fn run_container(
    image: String,
    name: Option<String>,
    ports: Option<String>,
    env_vars: Option<Vec<String>>,
) -> Result<String, String> {
    let mut args: Vec<String> = vec!["run".into(), "-d".into()];

    if let Some(n) = name {
        if !n.is_empty() {
            args.push("--name".into());
            args.push(n);
        }
    }

    if let Some(p) = ports {
        for mapping in p.split(',') {
            let mapping = mapping.trim();
            if !mapping.is_empty() {
                args.push("-p".into());
                args.push(mapping.to_string());
            }
        }
    }

    if let Some(envs) = env_vars {
        for e in envs {
            if !e.is_empty() {
                args.push("-e".into());
                args.push(e);
            }
        }
    }

    args.push(image);

    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    CliExecutor::run(DOCKER, &refs).await
}

#[tauri::command]
pub async fn stream_container_logs(app: AppHandle, id: String) -> Result<(), String> {
    let docker_host = crate::cli::executor::docker_host();

    let mut child = Command::new(DOCKER)
        .args(["logs", "-f", "--tail", "200", &id])
        .env("DOCKER_HOST", &docker_host)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker logs: {}", e))?;

    let stdout = child.stdout.take().ok_or("No stdout")?;
    let stderr = child.stderr.take().ok_or("No stderr")?;
    let event_name = format!("container-log-{}", id);

    let app_clone = app.clone();
    let event_clone = event_name.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit(&event_clone, &line);
        }
    });

    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app.emit(&event_name, &line);
        }
    });

    Ok(())
}
