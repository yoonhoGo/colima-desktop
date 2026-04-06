use crate::cli::executor::CliExecutor;
use crate::cli::types::{ColimaListEntry, HostInfo, VmSettings};

#[tauri::command]
pub async fn get_vm_settings() -> Result<VmSettings, String> {
    let stdout = CliExecutor::run("colima", &["list", "--json"]).await?;

    // colima list --json outputs one JSON object per line
    let entry: ColimaListEntry = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .next()
        .ok_or("No colima instance found".to_string())
        .and_then(|line| {
            serde_json::from_str(line)
                .map_err(|e| format!("Failed to parse colima list: {}", e))
        })?;

    Ok(VmSettings {
        cpus: entry.cpus,
        memory_gib: entry.memory as f64 / 1_073_741_824.0,
        disk_gib: entry.disk as f64 / 1_073_741_824.0,
        runtime: entry.runtime,
        network_address: entry.network_address,
    })
}

#[tauri::command]
pub async fn get_host_info() -> Result<HostInfo, String> {
    let cpu_str = CliExecutor::run("sysctl", &["-n", "hw.ncpu"]).await?;
    let cpus: u32 = cpu_str
        .trim()
        .parse()
        .map_err(|e| format!("Failed to parse CPU count: {}", e))?;

    let mem_str = CliExecutor::run("sysctl", &["-n", "hw.memsize"]).await?;
    let mem_bytes: u64 = mem_str
        .trim()
        .parse()
        .map_err(|e| format!("Failed to parse memory: {}", e))?;

    Ok(HostInfo {
        cpus,
        memory_gib: mem_bytes as f64 / 1_073_741_824.0,
    })
}

#[tauri::command]
pub async fn apply_vm_settings(
    cpus: u32,
    memory_gib: u32,
    disk_gib: u32,
    runtime: String,
    network_address: String,
) -> Result<(), String> {
    // Stop if running (ignore error if already stopped)
    CliExecutor::run("colima", &["stop"]).await.ok();

    let cpu_str = cpus.to_string();
    let mem_str = memory_gib.to_string();
    let disk_str = disk_gib.to_string();

    let mut args = vec![
        "start",
        "--cpu", &cpu_str,
        "--memory", &mem_str,
        "--disk", &disk_str,
        "--runtime", &runtime,
    ];

    if !network_address.is_empty() {
        args.push("--network-address");
        args.push(&network_address);
    }

    CliExecutor::run("colima", &args).await?;
    Ok(())
}
