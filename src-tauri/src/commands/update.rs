use crate::cli::executor::CliExecutor;
use crate::cli::types::{ColimaVersion, RuntimeVersion, VersionCheck};

fn parse_colima_version(output: &str) -> Result<ColimaVersion, String> {
    let lines: Vec<&str> = output.lines().collect();

    let version = lines
        .first()
        .and_then(|line| line.strip_prefix("colima version "))
        .map(|v| v.trim().to_string())
        .ok_or_else(|| "Failed to parse colima version".to_string())?;

    let git_commit = lines
        .iter()
        .find_map(|line| line.trim().strip_prefix("git commit: "))
        .map(|v| v.trim().to_string())
        .unwrap_or_default();

    let mut runtime_versions = Vec::new();
    let mut in_runtime_section = false;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if in_runtime_section {
                in_runtime_section = false;
            }
            continue;
        }
        if trimmed.starts_with("runtime:") {
            in_runtime_section = true;
            continue;
        }
        // Indented lines belong to the current section
        if in_runtime_section && (line.starts_with(' ') || line.starts_with('\t')) {
            if let Some((name, ver)) = trimmed.split_once(':') {
                runtime_versions.push(RuntimeVersion {
                    name: name.trim().to_string(),
                    version: ver.trim().to_string(),
                });
            }
        } else if in_runtime_section {
            // Non-indented, non-empty line means new section
            in_runtime_section = false;
        }
    }

    Ok(ColimaVersion {
        version,
        git_commit,
        runtime_versions,
    })
}

#[tauri::command]
pub async fn get_colima_version() -> Result<ColimaVersion, String> {
    let output = CliExecutor::run("colima", &["version"]).await?;
    parse_colima_version(&output)
}

#[tauri::command]
pub async fn update_colima_runtime() -> Result<String, String> {
    let output = CliExecutor::run("colima", &["update"]).await?;
    Ok(output.trim().to_string())
}

#[tauri::command]
pub async fn check_latest_version() -> Result<VersionCheck, String> {
    let version_output = CliExecutor::run("colima", &["version"]).await?;
    let colima_version = parse_colima_version(&version_output)?;
    let current = colima_version.version.clone();

    let curl_output = CliExecutor::run(
        "curl",
        &["-s", "https://api.github.com/repos/abiosoft/colima/releases/latest"],
    )
    .await?;

    let json: serde_json::Value = serde_json::from_str(&curl_output)
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let latest = json["tag_name"]
        .as_str()
        .map(|s| s.strip_prefix('v').unwrap_or(s).to_string())
        .ok_or_else(|| "Failed to get latest version from GitHub".to_string())?;

    let current_clean = current.strip_prefix('v').unwrap_or(&current).to_string();
    let update_available = current_clean != latest;

    Ok(VersionCheck {
        current: current_clean,
        latest,
        update_available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_colima_version() {
        let output = "\
colima version 0.8.1
git commit: abc1234

runtime: docker
  containerd: v2.0.0
  runc: v1.2.0

image:
  ubuntu: 24.04";

        let result = parse_colima_version(output).unwrap();
        assert_eq!(result.version, "0.8.1");
        assert_eq!(result.git_commit, "abc1234");
        assert_eq!(result.runtime_versions.len(), 2);
        assert_eq!(result.runtime_versions[0].name, "containerd");
        assert_eq!(result.runtime_versions[0].version, "v2.0.0");
        assert_eq!(result.runtime_versions[1].name, "runc");
        assert_eq!(result.runtime_versions[1].version, "v1.2.0");
    }
}
