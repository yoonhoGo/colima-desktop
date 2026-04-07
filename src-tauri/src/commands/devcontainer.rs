use crate::cli::executor::{docker_host, CliExecutor};
use crate::cli::types::{
    DevContainerProject, DevContainerProjectWithStatus, DevContainerProjectsConfig,
    DevContainerReadConfig,
};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

const DOCKER: &str = "/opt/homebrew/bin/docker";

fn config_path() -> Result<std::path::PathBuf, String> {
    let config_dir = dirs::config_dir().ok_or("Cannot find config directory")?;
    let app_dir = config_dir.join("colima-desktop");
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    Ok(app_dir.join("devcontainer-projects.json"))
}

fn load_projects() -> Result<Vec<DevContainerProject>, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    let config: DevContainerProjectsConfig =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;
    Ok(config.projects)
}

fn save_projects(projects: &[DevContainerProject]) -> Result<(), String> {
    let path = config_path()?;
    let config = DevContainerProjectsConfig {
        projects: projects.to_vec(),
    };
    let content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

fn find_devcontainer_cli() -> Option<String> {
    let candidates = [
        "/opt/homebrew/bin/devcontainer",
        "/usr/local/bin/devcontainer",
    ];
    for path in candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    if let Ok(output) = std::process::Command::new("which")
        .arg("devcontainer")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }
    None
}

#[tauri::command]
pub async fn check_devcontainer_cli() -> Result<bool, String> {
    Ok(find_devcontainer_cli().is_some())
}

#[tauri::command]
pub async fn list_devcontainer_projects() -> Result<Vec<DevContainerProjectWithStatus>, String> {
    let projects = load_projects()?;
    let mut result = Vec::new();

    for project in projects {
        if !std::path::Path::new(&project.workspace_path).exists() {
            result.push(DevContainerProjectWithStatus {
                id: project.id,
                workspace_path: project.workspace_path,
                name: project.name,
                status: "path_missing".to_string(),
                container_id: None,
            });
            continue;
        }

        let label_filter = format!(
            "label=devcontainer.local_folder={}",
            project.workspace_path
        );
        let output = CliExecutor::run(
            DOCKER,
            &[
                "ps",
                "-a",
                "--filter",
                &label_filter,
                "--format",
                "{{.ID}}|{{.State}}",
            ],
        )
        .await;

        let (status, container_id) = match output {
            Ok(out) => {
                let line = out.trim();
                if line.is_empty() {
                    ("not_built".to_string(), None)
                } else {
                    let parts: Vec<&str> = line.lines().next().unwrap_or("").split('|').collect();
                    let cid = parts.first().map(|s| s.to_string());
                    let state = parts.get(1).unwrap_or(&"unknown");
                    let status = if *state == "running" {
                        "running".to_string()
                    } else {
                        "stopped".to_string()
                    };
                    (status, cid)
                }
            }
            Err(_) => ("not_built".to_string(), None),
        };

        result.push(DevContainerProjectWithStatus {
            id: project.id,
            workspace_path: project.workspace_path,
            name: project.name,
            status,
            container_id,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn add_devcontainer_project(workspace_path: String) -> Result<DevContainerProjectWithStatus, String> {
    let path = std::path::Path::new(&workspace_path);

    let has_config = path.join(".devcontainer").join("devcontainer.json").exists()
        || path.join(".devcontainer.json").exists();

    if !has_config {
        return Err("No devcontainer.json found in this project. Expected .devcontainer/devcontainer.json or .devcontainer.json".to_string());
    }

    let mut projects = load_projects()?;

    if projects.iter().any(|p| p.workspace_path == workspace_path) {
        return Err("This project is already registered".to_string());
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project = DevContainerProject {
        id: uuid::Uuid::new_v4().to_string(),
        workspace_path: workspace_path.clone(),
        name: name.clone(),
    };

    projects.push(project.clone());
    save_projects(&projects)?;

    Ok(DevContainerProjectWithStatus {
        id: project.id,
        workspace_path,
        name,
        status: "not_built".to_string(),
        container_id: None,
    })
}

#[tauri::command]
pub async fn remove_devcontainer_project(id: String, remove_container: bool) -> Result<(), String> {
    let mut projects = load_projects()?;

    let project = projects
        .iter()
        .find(|p| p.id == id)
        .ok_or("Project not found")?
        .clone();

    if remove_container {
        let label_filter = format!(
            "label=devcontainer.local_folder={}",
            project.workspace_path
        );
        let output = CliExecutor::run(
            DOCKER,
            &[
                "ps",
                "-a",
                "--filter",
                &label_filter,
                "--format",
                "{{.ID}}",
            ],
        )
        .await;

        if let Ok(out) = output {
            for cid in out.lines() {
                let cid = cid.trim();
                if !cid.is_empty() {
                    let _ = CliExecutor::run(DOCKER, &["rm", "-f", cid]).await;
                }
            }
        }
    }

    projects.retain(|p| p.id != id);
    save_projects(&projects)?;
    Ok(())
}

#[tauri::command]
pub async fn devcontainer_up(app: AppHandle, workspace_path: String) -> Result<(), String> {
    let cli = find_devcontainer_cli().ok_or(
        "devcontainer CLI not found. Install with: npm install -g @devcontainers/cli",
    )?;

    let event_name = format!("devcontainer-log-{}", workspace_path.replace('/', "_"));
    let docker_host_val = docker_host();

    let mut child = Command::new(&cli)
        .args(["up", "--workspace-folder", &workspace_path])
        .env("DOCKER_HOST", &docker_host_val)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn devcontainer up: {}", e))?;

    let stdout = child.stdout.take().ok_or("No stdout")?;
    let stderr = child.stderr.take().ok_or("No stderr")?;

    let app_clone = app.clone();
    let event_clone = event_name.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit(&event_clone, &line);
        }
    });

    let app_clone2 = app.clone();
    let event_clone2 = event_name.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone2.emit(&event_clone2, &line);
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for devcontainer up: {}", e))?;

    if !status.success() {
        return Err("devcontainer up failed. Check the build log for details.".to_string());
    }

    let _ = app.emit(&event_name, "[done]");
    Ok(())
}

#[tauri::command]
pub async fn devcontainer_build(app: AppHandle, workspace_path: String) -> Result<(), String> {
    let cli = find_devcontainer_cli().ok_or(
        "devcontainer CLI not found. Install with: npm install -g @devcontainers/cli",
    )?;

    let event_name = format!("devcontainer-log-{}", workspace_path.replace('/', "_"));
    let docker_host_val = docker_host();

    let mut child = Command::new(&cli)
        .args(["build", "--workspace-folder", &workspace_path])
        .env("DOCKER_HOST", &docker_host_val)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn devcontainer build: {}", e))?;

    let stdout = child.stdout.take().ok_or("No stdout")?;
    let stderr = child.stderr.take().ok_or("No stderr")?;

    let app_clone = app.clone();
    let event_clone = event_name.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit(&event_clone, &line);
        }
    });

    let app_clone2 = app.clone();
    let event_clone2 = event_name.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone2.emit(&event_clone2, &line);
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for devcontainer build: {}", e))?;

    if !status.success() {
        return Err("devcontainer build failed. Check the build log for details.".to_string());
    }

    let _ = app.emit(&event_name, "[done]");
    Ok(())
}

#[tauri::command]
pub async fn devcontainer_stop(workspace_path: String) -> Result<(), String> {
    let label_filter = format!(
        "label=devcontainer.local_folder={}",
        workspace_path
    );
    let output = CliExecutor::run(
        DOCKER,
        &[
            "ps",
            "-q",
            "--filter",
            &label_filter,
        ],
    )
    .await?;

    for cid in output.lines() {
        let cid = cid.trim();
        if !cid.is_empty() {
            CliExecutor::run(DOCKER, &["stop", cid]).await?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn devcontainer_read_config(workspace_path: String) -> Result<DevContainerReadConfig, String> {
    let cli = find_devcontainer_cli().ok_or(
        "devcontainer CLI not found. Install with: npm install -g @devcontainers/cli",
    )?;

    let docker_host_val = docker_host();
    let output = Command::new(&cli)
        .args(["read-configuration", "--workspace-folder", &workspace_path])
        .env("DOCKER_HOST", &docker_host_val)
        .output()
        .await
        .map_err(|e| format!("Failed to run read-configuration: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("read-configuration failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| format!("JSON parse error: {}", e))?;

    let config = &parsed["configuration"];

    let image = config["image"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let features = config["features"]
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    let forward_ports = config["forwardPorts"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|n| n as u16))
                .collect()
        })
        .unwrap_or_default();

    let remote_user = config["remoteUser"]
        .as_str()
        .unwrap_or("root")
        .to_string();

    Ok(DevContainerReadConfig {
        image,
        features,
        forward_ports,
        remote_user,
    })
}
