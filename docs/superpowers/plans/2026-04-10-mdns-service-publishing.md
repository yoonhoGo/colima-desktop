# mDNS Service Publishing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Colima 내 실행 중인 Docker 컨테이너를 LAN에 mDNS 서비스로 자동 노출하여 `container-name.local`로 접근 가능하게 한다.

**Architecture:** Rust `mdns-sd` 크레이트로 Tauri 프로세스 내에서 mDNS 데몬을 직접 운영. `MdnsManager`를 Tauri State로 관리하고, 5초 폴링으로 컨테이너와 mDNS 등록을 동기화. 설정은 Tauri 앱 데이터 경로에 JSON으로 영속화.

**Tech Stack:** Rust (mdns-sd, local-ip-address, hostname), Tauri 2 State, React 19, TanStack React Query, shadcn/ui

**Spec:** `docs/superpowers/specs/2026-04-10-mdns-service-publishing-design.md`

---

## File Structure

### New Files

| File | Responsibility |
|------|---------------|
| `src-tauri/src/mdns/mod.rs` | mDNS 모듈 공개 인터페이스 |
| `src-tauri/src/mdns/config.rs` | MdnsConfig 타입 + JSON 영속화 (load/save) |
| `src-tauri/src/mdns/manager.rs` | MdnsManager — ServiceDaemon 래핑, 서비스 등록/해제 |
| `src-tauri/src/mdns/sync.rs` | 컨테이너 ↔ mDNS 동기화 로직 + 포트 파싱 |
| `src-tauri/src/commands/mdns.rs` | Tauri IPC 커맨드 핸들러 |
| `src/hooks/useMdns.ts` | React Query 훅 (조회, 폴링, mutation) |
| `src/components/containers/ContainerMdnsBadge.tsx` | 컨테이너 행 내 mDNS 상태 배지 |
| `src/components/containers/ContainerMdnsDialog.tsx` | 컨테이너별 오버라이드 설정 다이얼로그 |
| `src/components/settings/MdnsSettings.tsx` | 글로벌 mDNS 설정 패널 |

### Modified Files

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | mdns-sd, local-ip-address, hostname 의존성 추가 |
| `src-tauri/src/lib.rs` | mdns 모듈 선언, MdnsManager State 등록, 커맨드 핸들러 등록 |
| `src-tauri/src/commands/mod.rs` | `pub mod mdns;` 추가 |
| `src/types/index.ts` | mDNS 관련 타입 추가 |
| `src/lib/tauri.ts` | mDNS API 래퍼 추가 |
| `src/components/containers/ContainerRow.tsx` | ContainerMdnsBadge 통합 |
| `src/components/layout/MainLayout.tsx` | Settings 탭에 mDNS 추가 |

---

## Task 1: Rust 의존성 추가

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Cargo.toml에 mDNS 관련 의존성 추가**

`src-tauri/Cargo.toml`의 `[dependencies]` 섹션 끝에 추가:

```toml
mdns-sd = { version = "0.11", features = ["async"] }
local-ip-address = "0.6"
hostname = "0.4"
```

- [ ] **Step 2: 빌드 확인**

Run: `cd src-tauri && cargo check`
Expected: 의존성 다운로드 후 성공

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat(mdns): add mdns-sd, local-ip-address, hostname dependencies"
```

---

## Task 2: mDNS Config 타입 및 영속화

**Files:**
- Create: `src-tauri/src/mdns/mod.rs`
- Create: `src-tauri/src/mdns/config.rs`

- [ ] **Step 1: mdns 모듈 엔트리 파일 생성**

`src-tauri/src/mdns/mod.rs`:

```rust
pub mod config;
pub mod manager;
pub mod sync;
```

- [ ] **Step 2: config.rs 작성 — 타입 정의**

`src-tauri/src/mdns/config.rs`:

```rust
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
```

- [ ] **Step 3: 테스트 실행**

Run: `cd src-tauri && cargo test mdns::config`
Expected: 3개 테스트 PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/mdns/
git commit -m "feat(mdns): add MdnsConfig types and persistence logic"
```

---

## Task 3: MdnsManager 구현

**Files:**
- Create: `src-tauri/src/mdns/manager.rs`

- [ ] **Step 1: manager.rs 작성**

`src-tauri/src/mdns/manager.rs`:

```rust
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct RegisteredService {
    pub container_name: String,
    pub hostname: String,
    pub service_type: String,
    pub port: u16,
    pub fullname: String,
    pub auto_registered: bool,
}

pub struct MdnsManagerInner {
    daemon: Option<ServiceDaemon>,
    pub registered: HashMap<String, RegisteredService>,
}

pub type MdnsManager = Arc<Mutex<MdnsManagerInner>>;

pub fn create_mdns_manager() -> MdnsManager {
    Arc::new(Mutex::new(MdnsManagerInner {
        daemon: None,
        registered: HashMap::new(),
    }))
}

impl MdnsManagerInner {
    pub fn enable(&mut self) -> Result<(), String> {
        if self.daemon.is_some() {
            return Ok(());
        }
        let daemon = ServiceDaemon::new()
            .map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
        self.daemon = Some(daemon);
        Ok(())
    }

    pub fn disable(&mut self) {
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.shutdown();
        }
        self.registered.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.daemon.is_some()
    }

    pub fn register(
        &mut self,
        container_name: &str,
        hostname: &str,
        service_type: &str,
        port: u16,
        auto_registered: bool,
    ) -> Result<(), String> {
        let daemon = self.daemon.as_ref().ok_or("mDNS daemon not running")?;

        let stype = normalize_service_type(service_type);
        let host_ip = local_ip_address::local_ip()
            .map_err(|e| format!("Failed to get local IP: {}", e))?;
        let sys_hostname = hostname::get()
            .map_err(|e| format!("Failed to get hostname: {}", e))?
            .to_string_lossy()
            .to_string();
        let fqdn_host = if sys_hostname.ends_with('.') {
            sys_hostname
        } else {
            format!("{}.", sys_hostname)
        };

        let fullname = format!("{}.{}", hostname, stype);

        let service = ServiceInfo::new(
            &stype,
            hostname,
            &fqdn_host,
            host_ip,
            port,
            [("source", "colima-desktop")].as_ref(),
        )
        .map_err(|e| format!("Failed to create ServiceInfo: {}", e))?;

        daemon
            .register(service)
            .map_err(|e| format!("Failed to register service: {}", e))?;

        self.registered.insert(
            container_name.to_string(),
            RegisteredService {
                container_name: container_name.to_string(),
                hostname: hostname.to_string(),
                service_type: stype.clone(),
                port,
                fullname,
                auto_registered,
            },
        );

        Ok(())
    }

    pub fn unregister(&mut self, container_name: &str) -> Result<(), String> {
        if let Some(service) = self.registered.remove(container_name) {
            if let Some(daemon) = &self.daemon {
                let _ = daemon.unregister(&service.fullname);
            }
        }
        Ok(())
    }

    pub fn list_services(&self) -> Vec<RegisteredService> {
        self.registered.values().cloned().collect()
    }
}

pub fn normalize_service_type(service_type: &str) -> String {
    if service_type.ends_with(".local.") {
        service_type.to_string()
    } else if service_type.ends_with('.') {
        format!("{}local.", service_type)
    } else {
        format!("{}.local.", service_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_service_type() {
        assert_eq!(
            normalize_service_type("_http._tcp"),
            "_http._tcp.local."
        );
        assert_eq!(
            normalize_service_type("_http._tcp."),
            "_http._tcp.local."
        );
        assert_eq!(
            normalize_service_type("_http._tcp.local."),
            "_http._tcp.local."
        );
    }
}
```

- [ ] **Step 2: 테스트 실행**

Run: `cd src-tauri && cargo test mdns::manager`
Expected: 1개 테스트 PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/mdns/manager.rs
git commit -m "feat(mdns): add MdnsManager with service registration/unregistration"
```

---

## Task 4: 동기화 로직

**Files:**
- Create: `src-tauri/src/mdns/sync.rs`

- [ ] **Step 1: sync.rs 작성**

`src-tauri/src/mdns/sync.rs`:

```rust
use crate::cli::executor::CliExecutor;
use crate::cli::types::DockerPsEntry;
use crate::mdns::config::{ContainerMdnsOverride, MdnsConfig};
use crate::mdns::manager::{MdnsManagerInner, RegisteredService};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

const DOCKER: &str = "docker";

#[derive(Debug, Serialize, Clone)]
pub struct MdnsSyncResult {
    pub services: Vec<MdnsServiceEntry>,
    pub daemon_running: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct MdnsServiceEntry {
    pub container_id: String,
    pub container_name: String,
    pub hostname: String,
    pub service_type: String,
    pub port: u16,
    pub registered: bool,
    pub auto_registered: bool,
}

pub async fn sync_containers(
    manager: &mut MdnsManagerInner,
    config: &MdnsConfig,
) -> Result<MdnsSyncResult, String> {
    if !manager.is_enabled() {
        return Ok(MdnsSyncResult {
            services: vec![],
            daemon_running: false,
        });
    }

    // 1. 현재 실행 중인 컨테이너 목록 조회
    let entries: Vec<DockerPsEntry> = match CliExecutor::run_json_lines(
        DOCKER,
        &["ps", "--format", "{{json .}}"],
    )
    .await
    {
        Ok(e) => e,
        Err(_) => {
            // Docker/Colima가 실행 중이 아니면 빈 결과 반환
            return Ok(MdnsSyncResult {
                services: manager
                    .list_services()
                    .iter()
                    .map(|s| MdnsServiceEntry {
                        container_id: String::new(),
                        container_name: s.container_name.clone(),
                        hostname: s.hostname.clone(),
                        service_type: s.service_type.clone(),
                        port: s.port,
                        registered: true,
                        auto_registered: s.auto_registered,
                    })
                    .collect(),
                daemon_running: true,
            });
        }
    };

    // 2. 실행 중인 컨테이너를 Map으로 변환
    let running: HashMap<String, &DockerPsEntry> = entries
        .iter()
        .map(|e| (e.names.clone(), e))
        .collect();

    // 3. 종료된 컨테이너의 서비스 해제
    let registered_names: Vec<String> = manager.registered.keys().cloned().collect();
    for name in &registered_names {
        if !running.contains_key(name) {
            let _ = manager.unregister(name);
        }
    }

    // 4. 실행 중인 컨테이너 처리
    let mut result_services = Vec::new();
    let mut processed: HashSet<String> = HashSet::new();

    for entry in &entries {
        let name = &entry.names;
        processed.insert(name.clone());

        // 오버라이드 확인
        if let Some(ovr) = config.container_overrides.get(name) {
            if !ovr.enabled {
                // 비활성화된 오버라이드 → 서비스 해제
                if manager.registered.contains_key(name) {
                    let _ = manager.unregister(name);
                }
                result_services.push(build_entry(entry, ovr, config, false, false));
                continue;
            }

            let hostname = ovr.hostname.as_deref().unwrap_or(name);
            let stype = ovr
                .service_type
                .as_deref()
                .unwrap_or(&config.default_service_type);
            let port = ovr.port.or_else(|| parse_first_host_port(&entry.ports));

            if let Some(port) = port {
                if !manager.registered.contains_key(name) {
                    let _ = manager.register(name, hostname, stype, port, false);
                }
                result_services.push(MdnsServiceEntry {
                    container_id: entry.id.clone(),
                    container_name: name.clone(),
                    hostname: hostname.to_string(),
                    service_type: stype.to_string(),
                    port,
                    registered: manager.registered.contains_key(name),
                    auto_registered: false,
                });
            } else {
                result_services.push(build_entry(entry, ovr, config, false, false));
            }
        } else if config.auto_register {
            // 자동 등록
            if let Some(port) = parse_first_host_port(&entry.ports) {
                if !manager.registered.contains_key(name) {
                    let _ = manager.register(
                        name,
                        name,
                        &config.default_service_type,
                        port,
                        true,
                    );
                }
                result_services.push(MdnsServiceEntry {
                    container_id: entry.id.clone(),
                    container_name: name.clone(),
                    hostname: name.clone(),
                    service_type: config.default_service_type.clone(),
                    port,
                    registered: manager.registered.contains_key(name),
                    auto_registered: true,
                });
            }
            // 포트 미노출 컨테이너는 자동 등록 대상에서 제외
        }
    }

    Ok(MdnsSyncResult {
        services: result_services,
        daemon_running: true,
    })
}

fn build_entry(
    entry: &DockerPsEntry,
    ovr: &ContainerMdnsOverride,
    config: &MdnsConfig,
    registered: bool,
    auto_registered: bool,
) -> MdnsServiceEntry {
    MdnsServiceEntry {
        container_id: entry.id.clone(),
        container_name: entry.names.clone(),
        hostname: ovr
            .hostname
            .clone()
            .unwrap_or_else(|| entry.names.clone()),
        service_type: ovr
            .service_type
            .clone()
            .unwrap_or_else(|| config.default_service_type.clone()),
        port: ovr.port.unwrap_or(0),
        registered,
        auto_registered,
    }
}

pub fn parse_first_host_port(ports: &str) -> Option<u16> {
    // Format: "0.0.0.0:8080->80/tcp, :::8080->80/tcp"
    let segment = ports.split(',').next()?;
    let arrow = segment.find("->")?;
    let before_arrow = &segment[..arrow];
    let colon_pos = before_arrow.rfind(':')?;
    let port_str = &before_arrow[colon_pos + 1..];
    port_str.trim().parse::<u16>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_first_host_port_standard() {
        assert_eq!(parse_first_host_port("0.0.0.0:8080->80/tcp"), Some(8080));
    }

    #[test]
    fn test_parse_first_host_port_multiple() {
        assert_eq!(
            parse_first_host_port("0.0.0.0:8080->80/tcp, :::8080->80/tcp"),
            Some(8080)
        );
    }

    #[test]
    fn test_parse_first_host_port_no_mapping() {
        assert_eq!(parse_first_host_port("80/tcp"), None);
    }

    #[test]
    fn test_parse_first_host_port_empty() {
        assert_eq!(parse_first_host_port(""), None);
    }

    #[test]
    fn test_parse_first_host_port_different_port() {
        assert_eq!(
            parse_first_host_port("0.0.0.0:3000->3000/tcp"),
            Some(3000)
        );
    }
}
```

- [ ] **Step 2: 테스트 실행**

Run: `cd src-tauri && cargo test mdns::sync`
Expected: 5개 테스트 PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/mdns/sync.rs
git commit -m "feat(mdns): add container-mDNS sync logic with port parsing"
```

---

## Task 5: Tauri 커맨드 핸들러 + 등록

**Files:**
- Create: `src-tauri/src/commands/mdns.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: commands/mdns.rs 작성**

`src-tauri/src/commands/mdns.rs`:

```rust
use crate::mdns::config::{self, ContainerMdnsOverride, MdnsConfig};
use crate::mdns::manager::MdnsManager;
use crate::mdns::sync::{self, MdnsSyncResult};
use std::path::PathBuf;
use tauri::{Manager, State};

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(dir.join("mdns-config.json"))
}

#[tauri::command]
pub async fn mdns_get_config(app: tauri::AppHandle) -> Result<MdnsConfig, String> {
    let path = config_path(&app)?;
    Ok(config::load_config(&path).await)
}

#[tauri::command]
pub async fn mdns_set_config(
    app: tauri::AppHandle,
    state: State<'_, MdnsManager>,
    config: MdnsConfig,
) -> Result<(), String> {
    let path = config_path(&app)?;

    let mut manager = state.lock().await;
    if config.enabled {
        manager.enable()?;
    } else {
        manager.disable();
    }

    config::save_config(&path, &config).await?;
    Ok(())
}

#[tauri::command]
pub async fn mdns_set_container_override(
    app: tauri::AppHandle,
    container_name: String,
    override_config: ContainerMdnsOverride,
) -> Result<(), String> {
    let path = config_path(&app)?;
    let mut config = config::load_config(&path).await;
    config
        .container_overrides
        .insert(container_name, override_config);
    config::save_config(&path, &config).await?;
    Ok(())
}

#[tauri::command]
pub async fn mdns_remove_container_override(
    app: tauri::AppHandle,
    container_name: String,
) -> Result<(), String> {
    let path = config_path(&app)?;
    let mut config = config::load_config(&path).await;
    config.container_overrides.remove(&container_name);
    config::save_config(&path, &config).await?;
    Ok(())
}

#[tauri::command]
pub async fn mdns_sync_containers(
    app: tauri::AppHandle,
    state: State<'_, MdnsManager>,
) -> Result<MdnsSyncResult, String> {
    let path = config_path(&app)?;
    let config = config::load_config(&path).await;
    let mut manager = state.lock().await;

    // enabled 상태 동기화
    if config.enabled && !manager.is_enabled() {
        manager.enable()?;
    } else if !config.enabled && manager.is_enabled() {
        manager.disable();
    }

    sync::sync_containers(&mut manager, &config).await
}

#[tauri::command]
pub async fn mdns_get_status(
    app: tauri::AppHandle,
    state: State<'_, MdnsManager>,
) -> Result<MdnsStatusResponse, String> {
    let path = config_path(&app)?;
    let config = config::load_config(&path).await;
    let manager = state.lock().await;
    Ok(MdnsStatusResponse {
        enabled: config.enabled,
        daemon_running: manager.is_enabled(),
        registered_count: manager.registered.len(),
        services: manager.list_services(),
    })
}

#[derive(serde::Serialize)]
pub struct MdnsStatusResponse {
    pub enabled: bool,
    pub daemon_running: bool,
    pub registered_count: usize,
    pub services: Vec<crate::mdns::manager::RegisteredService>,
}
```

- [ ] **Step 2: commands/mod.rs에 mdns 모듈 추가**

`src-tauri/src/commands/mod.rs`의 마지막 줄에 추가:

```rust
pub mod mdns;
```

- [ ] **Step 3: lib.rs에 mdns 모듈 선언 및 State/커맨드 등록**

`src-tauri/src/lib.rs`의 맨 위에 `mod mdns;` 추가:

```rust
mod cli;
mod commands;
mod mdns;
mod tray;
```

`tauri::Builder::default()` 체인에 `.manage()` 추가 (`.invoke_handler()` 직전):

```rust
        .manage(mdns::manager::create_mdns_manager())
```

`invoke_handler` 매크로 내부의 마지막 항목 뒤에 mDNS 커맨드 추가:

```rust
            // mDNS
            commands::mdns::mdns_get_config,
            commands::mdns::mdns_set_config,
            commands::mdns::mdns_set_container_override,
            commands::mdns::mdns_remove_container_override,
            commands::mdns::mdns_sync_containers,
            commands::mdns::mdns_get_status,
```

- [ ] **Step 4: 빌드 확인**

Run: `cd src-tauri && cargo check`
Expected: 성공 (컴파일 에러 없음)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/mdns.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(mdns): add Tauri command handlers and register State/commands"
```

---

## Task 6: 프론트엔드 타입 + API 래퍼

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: TypeScript 타입 추가**

`src/types/index.ts` 끝에 추가:

```typescript

// ─── mDNS ────────────────────────────────────────────────────────────────────

export interface MdnsConfig {
  enabled: boolean;
  auto_register: boolean;
  default_service_type: string;
  container_overrides: Record<string, ContainerMdnsOverride>;
}

export interface ContainerMdnsOverride {
  enabled: boolean;
  hostname?: string | null;
  service_type?: string | null;
  port?: number | null;
}

export interface MdnsServiceEntry {
  container_id: string;
  container_name: string;
  hostname: string;
  service_type: string;
  port: number;
  registered: boolean;
  auto_registered: boolean;
}

export interface MdnsSyncResult {
  services: MdnsServiceEntry[];
  daemon_running: boolean;
}

export interface MdnsStatusResponse {
  enabled: boolean;
  daemon_running: boolean;
  registered_count: number;
  services: MdnsRegisteredService[];
}

export interface MdnsRegisteredService {
  container_name: string;
  hostname: string;
  service_type: string;
  port: number;
  fullname: string;
  auto_registered: boolean;
}
```

- [ ] **Step 2: Tauri API 래퍼 추가**

`src/lib/tauri.ts`의 import 문에 mDNS 타입 추가:

```typescript
import type { ..., MdnsConfig, ContainerMdnsOverride, MdnsSyncResult, MdnsStatusResponse } from "../types";
```

`api` 객체의 마지막 항목 뒤에 추가:

```typescript

  // mDNS
  mdnsGetConfig: () =>
    invoke<MdnsConfig>("mdns_get_config"),
  mdnsSetConfig: (config: MdnsConfig) =>
    invoke<void>("mdns_set_config", { config }),
  mdnsSetContainerOverride: (containerName: string, overrideConfig: ContainerMdnsOverride) =>
    invoke<void>("mdns_set_container_override", { containerName, overrideConfig }),
  mdnsRemoveContainerOverride: (containerName: string) =>
    invoke<void>("mdns_remove_container_override", { containerName }),
  mdnsSyncContainers: () =>
    invoke<MdnsSyncResult>("mdns_sync_containers"),
  mdnsGetStatus: () =>
    invoke<MdnsStatusResponse>("mdns_get_status"),
```

- [ ] **Step 3: Commit**

```bash
git add src/types/index.ts src/lib/tauri.ts
git commit -m "feat(mdns): add frontend TypeScript types and Tauri API wrappers"
```

---

## Task 7: React Query 훅

**Files:**
- Create: `src/hooks/useMdns.ts`

- [ ] **Step 1: useMdns.ts 작성**

`src/hooks/useMdns.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";
import type { MdnsConfig, ContainerMdnsOverride } from "../types";

export function useMdnsConfig() {
  return useQuery({
    queryKey: ["mdns-config"],
    queryFn: api.mdnsGetConfig,
  });
}

export function useMdnsSync(enabled: boolean) {
  return useQuery({
    queryKey: ["mdns-sync"],
    queryFn: api.mdnsSyncContainers,
    refetchInterval: enabled ? 5000 : false,
    enabled,
  });
}

export function useMdnsStatus() {
  return useQuery({
    queryKey: ["mdns-status"],
    queryFn: api.mdnsGetStatus,
  });
}

export function useMdnsSetConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (config: MdnsConfig) => api.mdnsSetConfig(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-config"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-sync"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-status"] });
    },
  });
}

export function useMdnsSetOverride() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      containerName,
      overrideConfig,
    }: {
      containerName: string;
      overrideConfig: ContainerMdnsOverride;
    }) => api.mdnsSetContainerOverride(containerName, overrideConfig),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-config"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-sync"] });
    },
  });
}

export function useMdnsRemoveOverride() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (containerName: string) =>
      api.mdnsRemoveContainerOverride(containerName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-config"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-sync"] });
    },
  });
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/useMdns.ts
git commit -m "feat(mdns): add React Query hooks for mDNS config and sync"
```

---

## Task 8: ContainerMdnsBadge 컴포넌트

**Files:**
- Create: `src/components/containers/ContainerMdnsBadge.tsx`

- [ ] **Step 1: ContainerMdnsBadge.tsx 작성**

`src/components/containers/ContainerMdnsBadge.tsx`:

```tsx
import { Radio } from "lucide-react";
import type { MdnsServiceEntry } from "../../types";

interface ContainerMdnsBadgeProps {
  service: MdnsServiceEntry | undefined;
  onConfigure: () => void;
}

export function ContainerMdnsBadge({
  service,
  onConfigure,
}: ContainerMdnsBadgeProps) {
  if (!service) {
    return (
      <button
        onClick={onConfigure}
        className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
        title="Configure mDNS"
      >
        <Radio className="h-3 w-3" />
        <span>mDNS</span>
      </button>
    );
  }

  if (!service.registered) {
    return (
      <button
        onClick={onConfigure}
        className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
        title="mDNS disabled for this container"
      >
        <Radio className="h-3 w-3" />
        <span>mDNS off</span>
      </button>
    );
  }

  return (
    <button
      onClick={onConfigure}
      className="flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400 hover:text-emerald-700 dark:hover:text-emerald-300 transition-colors"
      title={`${service.hostname}:${service.port}`}
    >
      <Radio className="h-3 w-3" />
      <span className="font-mono">
        {service.hostname}:{service.port}
      </span>
      {service.auto_registered && (
        <span className="text-[10px] text-muted-foreground bg-muted px-1 rounded">
          auto
        </span>
      )}
    </button>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/containers/ContainerMdnsBadge.tsx
git commit -m "feat(mdns): add ContainerMdnsBadge component for inline mDNS status"
```

---

## Task 9: ContainerMdnsDialog 컴포넌트

**Files:**
- Create: `src/components/containers/ContainerMdnsDialog.tsx`

- [ ] **Step 1: ContainerMdnsDialog.tsx 작성**

`src/components/containers/ContainerMdnsDialog.tsx`:

```tsx
import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { X } from "lucide-react";
import type { ContainerMdnsOverride } from "../../types";
import { useMdnsSetOverride, useMdnsRemoveOverride } from "../../hooks/useMdns";

interface ContainerMdnsDialogProps {
  containerName: string;
  currentOverride: ContainerMdnsOverride | undefined;
  defaultServiceType: string;
  onClose: () => void;
}

export function ContainerMdnsDialog({
  containerName,
  currentOverride,
  defaultServiceType,
  onClose,
}: ContainerMdnsDialogProps) {
  const setOverride = useMdnsSetOverride();
  const removeOverride = useMdnsRemoveOverride();

  const [enabled, setEnabled] = useState(currentOverride?.enabled ?? true);
  const [hostname, setHostname] = useState(currentOverride?.hostname ?? "");
  const [serviceType, setServiceType] = useState(
    currentOverride?.service_type ?? ""
  );
  const [port, setPort] = useState(
    currentOverride?.port?.toString() ?? ""
  );

  useEffect(() => {
    if (currentOverride) {
      setEnabled(currentOverride.enabled);
      setHostname(currentOverride.hostname ?? "");
      setServiceType(currentOverride.service_type ?? "");
      setPort(currentOverride.port?.toString() ?? "");
    }
  }, [currentOverride]);

  const handleSave = () => {
    setOverride.mutate(
      {
        containerName,
        overrideConfig: {
          enabled,
          hostname: hostname.trim() || null,
          service_type: serviceType.trim() || null,
          port: port.trim() ? parseInt(port, 10) : null,
        },
      },
      { onSuccess: onClose }
    );
  };

  const handleRemove = () => {
    removeOverride.mutate(containerName, { onSuccess: onClose });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="glass-card w-full max-w-md p-6 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold">
            mDNS: {containerName}
          </h3>
          <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </div>

        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(e) => setEnabled(e.target.checked)}
            className="rounded"
          />
          Enable mDNS for this container
        </label>

        <div className="space-y-3">
          <div className="space-y-1">
            <label className="text-xs font-medium text-muted-foreground">
              Hostname
            </label>
            <Input
              placeholder={containerName}
              value={hostname}
              onChange={(e) => setHostname(e.target.value)}
              disabled={!enabled}
            />
            <p className="text-xs text-muted-foreground">
              Leave empty to use container name
            </p>
          </div>

          <div className="space-y-1">
            <label className="text-xs font-medium text-muted-foreground">
              Service Type
            </label>
            <Input
              placeholder={defaultServiceType}
              value={serviceType}
              onChange={(e) => setServiceType(e.target.value)}
              disabled={!enabled}
            />
          </div>

          <div className="space-y-1">
            <label className="text-xs font-medium text-muted-foreground">
              Port
            </label>
            <Input
              placeholder="Auto-detect"
              value={port}
              onChange={(e) => setPort(e.target.value)}
              type="number"
              disabled={!enabled}
            />
            <p className="text-xs text-muted-foreground">
              Leave empty to auto-detect from exposed ports
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 pt-2">
          <Button
            onClick={handleSave}
            disabled={setOverride.isPending}
            className="flex-1"
          >
            {setOverride.isPending ? "Saving..." : "Save"}
          </Button>
          {currentOverride && (
            <Button
              variant="outline"
              onClick={handleRemove}
              disabled={removeOverride.isPending}
              className="text-destructive"
            >
              Remove Override
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/containers/ContainerMdnsDialog.tsx
git commit -m "feat(mdns): add ContainerMdnsDialog for per-container override settings"
```

---

## Task 10: MdnsSettings 패널

**Files:**
- Create: `src/components/settings/MdnsSettings.tsx`

- [ ] **Step 1: MdnsSettings.tsx 작성**

`src/components/settings/MdnsSettings.tsx`:

```tsx
import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Loader2, Radio } from "lucide-react";
import { useMdnsConfig, useMdnsSetConfig, useMdnsStatus } from "../../hooks/useMdns";
import type { MdnsConfig } from "../../types";

export function MdnsSettings() {
  const { data: config, isLoading, error } = useMdnsConfig();
  const { data: status } = useMdnsStatus();
  const saveMutation = useMdnsSetConfig();

  const [enabled, setEnabled] = useState(false);
  const [autoRegister, setAutoRegister] = useState(true);
  const [defaultServiceType, setDefaultServiceType] = useState("_http._tcp.local.");

  useEffect(() => {
    if (config) {
      setEnabled(config.enabled);
      setAutoRegister(config.auto_register);
      setDefaultServiceType(config.default_service_type);
    }
  }, [config]);

  const hasChanges = (() => {
    if (!config) return false;
    return (
      enabled !== config.enabled ||
      autoRegister !== config.auto_register ||
      defaultServiceType !== config.default_service_type
    );
  })();

  const handleSave = () => {
    if (!config) return;
    const updated: MdnsConfig = {
      ...config,
      enabled,
      auto_register: autoRegister,
      default_service_type: defaultServiceType,
    };
    saveMutation.mutate(updated);
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center text-sm">
        Failed to load mDNS settings
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <div className="flex items-center gap-2">
        <Radio className="h-5 w-5" />
        <h2 className="text-lg font-semibold">mDNS Settings</h2>
      </div>

      <div className="space-y-5">
        <label className="flex items-center gap-2 text-sm font-medium">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(e) => setEnabled(e.target.checked)}
            disabled={saveMutation.isPending}
            className="rounded"
          />
          Enable mDNS Service Publishing
        </label>

        <p className="text-xs text-muted-foreground -mt-3">
          Expose running containers on your local network via mDNS (Bonjour)
        </p>

        <label className="flex items-center gap-2 text-sm font-medium">
          <input
            type="checkbox"
            checked={autoRegister}
            onChange={(e) => setAutoRegister(e.target.checked)}
            disabled={saveMutation.isPending || !enabled}
            className="rounded"
          />
          Auto-register containers with exposed ports
        </label>

        <div className="space-y-2">
          <label className="text-sm font-medium">Default Service Type</label>
          <Input
            value={defaultServiceType}
            onChange={(e) => setDefaultServiceType(e.target.value)}
            disabled={saveMutation.isPending || !enabled}
            placeholder="_http._tcp.local."
          />
        </div>

        {status && (
          <div className="glass-card p-3 space-y-1">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Daemon</span>
              <span className={status.daemon_running ? "text-emerald-600" : "text-muted-foreground"}>
                {status.daemon_running ? "Running" : "Stopped"}
              </span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Registered Services</span>
              <span>{status.registered_count}</span>
            </div>
          </div>
        )}
      </div>

      <div className="space-y-2">
        <Button
          onClick={handleSave}
          disabled={!hasChanges || saveMutation.isPending}
          className="w-full"
        >
          {saveMutation.isPending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Saving...
            </>
          ) : (
            "Save"
          )}
        </Button>

        {saveMutation.isError && (
          <p className="text-center text-xs text-destructive">
            Failed to save mDNS settings
          </p>
        )}

        {saveMutation.isSuccess && (
          <p className="text-center text-xs text-green-600">
            Settings saved successfully
          </p>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/settings/MdnsSettings.tsx
git commit -m "feat(mdns): add MdnsSettings panel for global mDNS configuration"
```

---

## Task 11: ContainerRow에 mDNS 배지 통합

**Files:**
- Modify: `src/components/containers/ContainerRow.tsx`

- [ ] **Step 1: ContainerRow에 mDNS 배지 및 다이얼로그 추가**

`src/components/containers/ContainerRow.tsx`를 수정:

import 섹션에 추가:
```typescript
import { useState } from "react";
import { ContainerMdnsBadge } from "./ContainerMdnsBadge";
import { ContainerMdnsDialog } from "./ContainerMdnsDialog";
import type { MdnsServiceEntry, ContainerMdnsOverride } from "../../types";
```

`ContainerRowProps` 인터페이스에 mDNS 프로퍼티 추가:
```typescript
interface ContainerRowProps {
  container: Container;
  onViewLogs: (id: string) => void;
  onInspect?: (id: string) => void;
  showServiceName?: boolean;
  mdnsService?: MdnsServiceEntry;
  mdnsOverride?: ContainerMdnsOverride;
  mdnsEnabled?: boolean;
  defaultServiceType?: string;
}
```

함수 시그니처 업데이트:
```typescript
export function ContainerRow({ container, onViewLogs, onInspect, showServiceName, mdnsService, mdnsOverride, mdnsEnabled, defaultServiceType }: ContainerRowProps) {
```

컴포넌트 내부에 다이얼로그 상태 추가:
```typescript
  const [showMdnsDialog, setShowMdnsDialog] = useState(false);
```

JSX의 포트 정보 표시 영역 (`.mt-1` div) 끝에 mDNS 배지 추가:
```tsx
          {mdnsEnabled && container.state === "running" && (
            <ContainerMdnsBadge
              service={mdnsService}
              onConfigure={() => setShowMdnsDialog(true)}
            />
          )}
```

컴포넌트의 return 문 끝에 (closing `</div>` 전) 다이얼로그 추가:
```tsx
      {showMdnsDialog && (
        <ContainerMdnsDialog
          containerName={container.name}
          currentOverride={mdnsOverride}
          defaultServiceType={defaultServiceType ?? "_http._tcp.local."}
          onClose={() => setShowMdnsDialog(false)}
        />
      )}
```

- [ ] **Step 2: ContainerList에서 mDNS 데이터를 ContainerRow로 전달**

`src/components/containers/ContainerList.tsx`를 수정:

import 섹션에 추가:
```typescript
import { useMdnsConfig, useMdnsSync } from "../../hooks/useMdns";
```

컴포넌트 내부에 mDNS 훅 추가 (기존 훅들 근처):
```typescript
  const { data: mdnsConfig } = useMdnsConfig();
  const { data: mdnsSync } = useMdnsSync(mdnsConfig?.enabled ?? false);
```

mDNS 서비스 룩업 헬퍼를 useMemo 등 근처에 추가:
```typescript
  const mdnsServiceMap = useMemo(() => {
    const map = new Map<string, typeof mdnsSync extends { services: (infer T)[] } | undefined ? T : never>();
    if (mdnsSync?.services) {
      for (const svc of mdnsSync.services) {
        map.set(svc.container_name, svc);
      }
    }
    return map;
  }, [mdnsSync]);
```

ContainerRow 렌더링 부분에 mDNS props 전달 (standalone 렌더링 및 ComposeGroup 내부):
```tsx
<ContainerRow
  key={container.id}
  container={container}
  onViewLogs={...}
  onInspect={...}
  mdnsService={mdnsServiceMap.get(container.name)}
  mdnsOverride={mdnsConfig?.container_overrides?.[container.name]}
  mdnsEnabled={mdnsConfig?.enabled}
  defaultServiceType={mdnsConfig?.default_service_type}
/>
```

참고: ContainerRow가 ComposeGroup을 통해 렌더링되는 경우, ComposeGroup 컴포넌트에도 mDNS 관련 props를 전달하고 내부 ContainerRow에 전파해야 합니다. ComposeGroup의 정확한 구조를 확인하여 적절히 수정하세요.

- [ ] **Step 3: 빌드 확인**

Run: `npm run build`
Expected: TypeScript 컴파일 + Vite 빌드 성공

- [ ] **Step 4: Commit**

```bash
git add src/components/containers/ContainerRow.tsx src/components/containers/ContainerList.tsx
git commit -m "feat(mdns): integrate mDNS badge and dialog into container list"
```

참고: ComposeGroup.tsx 수정이 필요한 경우 해당 파일도 staging에 포함하세요.

---

## Task 12: Settings에 mDNS 탭 추가

**Files:**
- Modify: `src/components/layout/MainLayout.tsx`

- [ ] **Step 1: MainLayout에 mDNS 설정 탭 추가**

`src/components/layout/MainLayout.tsx`를 수정:

import 섹션에 추가:
```typescript
import { MdnsSettings } from "../settings/MdnsSettings";
```

`SettingsTab` 타입에 `"mdns"` 추가:
```typescript
type SettingsTab = "vm" | "mounts" | "network" | "docker" | "mdns" | "terminal" | "update" | "appearance";
```

Settings 탭 버튼 영역에서 "Docker" 버튼과 "Terminal" 버튼 사이에 mDNS 버튼 추가:
```tsx
              <button
                onClick={() => setSettingsTab("mdns")}
                className={`flex-1 rounded-lg px-3 py-1.5 text-sm font-medium transition-all ${
                  settingsTab === "mdns"
                    ? "bg-[var(--glass-bg-active)] text-foreground shadow-sm border border-[var(--glass-border-strong)]"
                    : "text-muted-foreground hover:text-foreground border border-transparent"
                }`}
              >
                mDNS
              </button>
```

설정 컨텐츠 영역에서 Docker와 Terminal 사이에 추가:
```tsx
            {settingsTab === "mdns" && <MdnsSettings />}
```

- [ ] **Step 2: 빌드 확인**

Run: `npm run build`
Expected: 성공

- [ ] **Step 3: Commit**

```bash
git add src/components/layout/MainLayout.tsx
git commit -m "feat(mdns): add mDNS tab to Settings page"
```

---

## Task 13: 전체 통합 빌드 및 테스트

**Files:** (수정 없음 — 검증만)

- [ ] **Step 1: Rust 테스트 전체 실행**

Run: `cd src-tauri && cargo test`
Expected: 모든 테스트 PASS (config 3개, manager 1개, sync 5개 = 총 9개)

- [ ] **Step 2: Tauri 전체 빌드 확인**

Run: `npm run tauri build -- --debug`
Expected: 디버그 빌드 성공

실패 시 에러를 분석하고 수정합니다. `cargo check`와 `npm run build`를 각각 실행하여 Rust/프론트엔드 중 어디에서 문제가 발생하는지 확인합니다.

- [ ] **Step 3: 수동 기능 테스트**

다음을 확인:
1. Settings → mDNS 탭에서 Enable mDNS 토글 가능
2. mDNS 활성화 후 Containers 탭에서 mDNS 배지 표시 확인
3. 컨테이너 mDNS 배지 클릭 시 오버라이드 다이얼로그 표시
4. 다이얼로그에서 설정 변경 후 Save 동작 확인
5. 터미널에서 `dns-sd -B _http._tcp` (macOS) 또는 `avahi-browse _http._tcp` (Linux)으로 서비스 등록 확인

- [ ] **Step 4: 최종 Commit (필요 시)**

수정 사항이 있으면 커밋합니다.

```bash
git add -A
git commit -m "fix(mdns): resolve integration build issues"
```
