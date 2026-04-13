use crate::proxy::server::{ProxyServer, RouteTable};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;
use tokio::sync::{Mutex, Notify};

/// Managed state holding the proxy's route table and shutdown handle.
pub struct ProxyState {
    pub routes: RouteTable,
    pub shutdown: Arc<Notify>,
    pub running: Arc<Mutex<bool>>,
    pub port: u16,
}

#[derive(Serialize)]
pub struct ProxyRoute {
    pub hostname: String,
    pub target_port: u16,
}

#[derive(Serialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub port: u16,
    pub routes: Vec<ProxyRoute>,
    pub pf_enabled: bool,
}

#[tauri::command]
pub async fn proxy_start(state: State<'_, ProxyState>) -> Result<(), String> {
    let mut running = state.running.lock().await;
    if *running {
        return Ok(());
    }

    let port = state.port;
    let routes = Arc::clone(&state.routes);
    let shutdown = Arc::clone(&state.shutdown);

    tokio::spawn(async move {
        let server = ProxyServer::with_shared(port, routes, shutdown);
        if let Err(e) = server.run().await {
            eprintln!("Proxy server error: {}", e);
        }
    });

    *running = true;
    Ok(())
}

#[tauri::command]
pub async fn proxy_stop(state: State<'_, ProxyState>) -> Result<(), String> {
    let mut running = state.running.lock().await;
    if !*running {
        return Ok(());
    }
    state.shutdown.notify_one();
    *running = false;
    Ok(())
}

#[tauri::command]
pub async fn proxy_get_status(state: State<'_, ProxyState>) -> Result<ProxyStatus, String> {
    let running = *state.running.lock().await;
    let table = state.routes.lock().await;
    let routes: Vec<ProxyRoute> = table
        .iter()
        .map(|(hostname, port)| ProxyRoute {
            hostname: hostname.clone(),
            target_port: *port,
        })
        .collect();

    let pf_enabled = check_pf_enabled().await;

    Ok(ProxyStatus {
        running,
        port: state.port,
        routes,
        pf_enabled,
    })
}

#[tauri::command]
pub async fn proxy_add_route(
    state: State<'_, ProxyState>,
    hostname: String,
    target_port: u16,
) -> Result<(), String> {
    let mut table = state.routes.lock().await;
    table.insert(hostname, target_port);
    Ok(())
}

#[tauri::command]
pub async fn proxy_remove_route(
    state: State<'_, ProxyState>,
    hostname: String,
) -> Result<(), String> {
    let mut table = state.routes.lock().await;
    table.remove(&hostname);
    Ok(())
}

// ─── pf Port Forwarding ────────────────────────────────────────────────────

#[tauri::command]
pub async fn proxy_enable_pf(state: State<'_, ProxyState>) -> Result<(), String> {
    let port = state.port;
    let rule = format!(
        "rdr pass on lo0 inet proto tcp from any to any port 80 -> 127.0.0.1 port {}",
        port
    );

    // Use osascript to request admin privileges
    let script = format!(
        r#"do shell script "echo '{}' | pfctl -ef -" with administrator privileges"#,
        rule
    );

    let output = tokio::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .await
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // pfctl may print "pf already enabled" which is fine
        if !stderr.contains("pf already enabled") && !stderr.is_empty() {
            return Err(format!("pfctl failed: {}", stderr));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn proxy_disable_pf() -> Result<(), String> {
    let script = r#"do shell script "pfctl -df /etc/pf.conf" with administrator privileges"#;

    let output = tokio::process::Command::new("osascript")
        .args(["-e", script])
        .output()
        .await
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("pf not enabled") && !stderr.is_empty() {
            return Err(format!("pfctl failed: {}", stderr));
        }
    }

    Ok(())
}

async fn check_pf_enabled() -> bool {
    let output = tokio::process::Command::new("pfctl")
        .args(["-sr"])
        .output()
        .await;

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("rdr pass") && stdout.contains("port 80")
        }
        Err(_) => false,
    }
}
