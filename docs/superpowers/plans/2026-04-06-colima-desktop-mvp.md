# Colima Desktop MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Colima를 GUI로 관리할 수 있는 macOS 데스크탑 앱 MVP (컨테이너/이미지 관리 + 시스템 트레이)

**Architecture:** Tauri v2 (Rust backend) + React/TypeScript (frontend). Rust에서 colima/docker CLI를 child process로 실행하고 JSON 출력을 파싱. TanStack Query로 주기적 polling. 시스템 트레이 상주.

**Tech Stack:** Tauri v2, React 19, TypeScript, Vite, Tailwind CSS, Shadcn/ui, TanStack Query, Rust (serde, tokio)

---

## File Structure

```
colima-desktop/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/                    # App icons
│   └── src/
│       ├── lib.rs                # Tauri plugin/command registration
│       ├── main.rs               # Entry point
│       ├── tray.rs               # System tray setup
│       ├── cli/
│       │   ├── mod.rs
│       │   ├── executor.rs       # Shell command runner
│       │   └── types.rs          # Docker/Colima JSON output types
│       └── commands/
│           ├── mod.rs
│           ├── colima.rs         # colima status/start/stop
│           ├── container.rs      # docker ps/start/stop/rm/logs
│           └── image.rs          # docker images/pull/rmi
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── App.css
│   ├── types/
│   │   └── index.ts              # Shared TS types
│   ├── lib/
│   │   └── tauri.ts              # Tauri invoke wrappers
│   ├── hooks/
│   │   ├── useColimaStatus.ts
│   │   ├── useContainers.ts
│   │   └── useImages.ts
│   └── components/
│       ├── layout/
│       │   ├── Sidebar.tsx
│       │   └── MainLayout.tsx
│       ├── containers/
│       │   ├── ContainerList.tsx
│       │   ├── ContainerRow.tsx
│       │   └── ContainerLogs.tsx
│       └── images/
│           ├── ImageList.tsx
│           ├── ImageRow.tsx
│           └── ImagePull.tsx
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
└── components.json              # shadcn/ui config
```

---

### Task 1: Project Scaffolding — Tauri v2 + React + TypeScript

**Files:**
- Create: entire project skeleton via `create-tauri-app`
- Modify: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`

- [ ] **Step 1: Scaffold Tauri v2 project**

```bash
cd /Users/yoonho.go/workspace/colima-desktop
npm create tauri-app@latest . -- --template react-ts --manager npm
```

If the directory is not empty, move the existing `docs/` folder aside first, scaffold, then move it back.

- [ ] **Step 2: Install frontend dependencies**

```bash
npm install @tanstack/react-query
npm install -D tailwindcss @tailwindcss/vite
```

- [ ] **Step 3: Configure Tailwind CSS with Vite**

Replace `vite.config.ts`:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

Replace the contents of `src/App.css` (or the main CSS entry) with:

```css
@import "tailwindcss";
```

- [ ] **Step 4: Setup Shadcn/ui**

```bash
npx shadcn@latest init -d
```

This creates `components.json` and sets up the `src/components/ui/` directory. Accept defaults.

- [ ] **Step 5: Add required Shadcn components**

```bash
npx shadcn@latest add button badge input scroll-area separator toast dialog
```

- [ ] **Step 6: Configure Tauri capabilities for shell and tray**

Create `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "tray:default",
    "tray:allow-set-icon",
    "tray:allow-set-tooltip"
  ]
}
```

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["process", "io-util"] }
```

- [ ] **Step 7: Verify the app builds and launches**

```bash
npm run tauri dev
```

Expected: A Tauri window opens with the default React page.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri v2 + React + TypeScript project

Includes Tailwind CSS, Shadcn/ui, TanStack Query setup."
```

---

### Task 2: Rust CLI Executor

**Files:**
- Create: `src-tauri/src/cli/mod.rs`
- Create: `src-tauri/src/cli/executor.rs`
- Modify: `src-tauri/src/lib.rs` (add mod declaration)

- [ ] **Step 1: Create CLI module declaration**

Create `src-tauri/src/cli/mod.rs`:

```rust
pub mod executor;
pub mod types;
```

- [ ] **Step 2: Implement the executor**

Create `src-tauri/src/cli/executor.rs`:

```rust
use std::process::Output;
use tokio::process::Command;

pub struct CliExecutor;

impl CliExecutor {
    /// Run a command and return stdout as String.
    /// Returns Err with stderr content on non-zero exit.
    pub async fn run(program: &str, args: &[&str]) -> Result<String, String> {
        let output: Output = Command::new(program)
            .args(args)
            .env("DOCKER_HOST", docker_host())
            .output()
            .await
            .map_err(|e| format!("Failed to execute {}: {}", program, e))?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| format!("Invalid UTF-8 output: {}", e))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("{} failed: {}", program, stderr.trim()))
        }
    }

    /// Run a command and return stdout line by line, parsing each as JSON.
    pub async fn run_json_lines<T: serde::de::DeserializeOwned>(
        program: &str,
        args: &[&str],
    ) -> Result<Vec<T>, String> {
        let stdout = Self::run(program, args).await?;
        let mut results = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let item: T = serde_json::from_str(trimmed)
                .map_err(|e| format!("JSON parse error: {} for line: {}", e, trimmed))?;
            results.push(item);
        }
        Ok(results)
    }
}

fn docker_host() -> String {
    std::env::var("DOCKER_HOST").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("unix://{}/.colima/default/docker.sock", home)
    })
}
```

- [ ] **Step 3: Register the cli module in lib.rs**

Add to the top of `src-tauri/src/lib.rs`:

```rust
mod cli;
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/cli/ src-tauri/src/lib.rs
git commit -m "feat: add CLI executor for running docker/colima commands"
```

---

### Task 3: Rust CLI Types

**Files:**
- Create: `src-tauri/src/cli/types.rs`

The types are based on actual `docker ps --format json` and `docker images --format json` output:

```
// docker ps --format json sample:
// {"Command":"...","CreatedAt":"...","ID":"b257ec692a20","Image":"nginx:alpine",
//  "Names":"test-nginx","Ports":"80/tcp","State":"running","Status":"Up 5 seconds"}
//
// docker images --format json sample:
// {"Containers":"2","CreatedAt":"...","ID":"4764f7b6826b","Repository":"mongodb/mongodb-atlas-local",
//  "Size":"2.11GB","Tag":"8.0.4"}
//
// colima status --json sample:
// {"display_name":"colima","arch":"aarch64","runtime":"docker","cpu":2,
//  "memory":2147483648,"disk":107374182400,"kubernetes":false}
```

- [ ] **Step 1: Create types file**

Create `src-tauri/src/cli/types.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Raw JSON output from `docker ps --format json`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DockerPsEntry {
    #[serde(rename = "ID")]
    pub id: String,
    pub names: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
    pub created_at: String,
}

/// Frontend-facing container type
#[derive(Debug, Serialize, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
    pub created_at: String,
}

impl From<DockerPsEntry> for Container {
    fn from(entry: DockerPsEntry) -> Self {
        Container {
            id: entry.id,
            name: entry.names,
            image: entry.image,
            state: entry.state,
            status: entry.status,
            ports: entry.ports,
            created_at: entry.created_at,
        }
    }
}

/// Raw JSON output from `docker images --format json`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DockerImageEntry {
    #[serde(rename = "ID")]
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    pub created_at: String,
    #[serde(default)]
    pub containers: String,
}

/// Frontend-facing image type
#[derive(Debug, Serialize, Clone)]
pub struct Image {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    pub created_at: String,
    pub in_use: bool,
}

impl From<DockerImageEntry> for Image {
    fn from(entry: DockerImageEntry) -> Self {
        let in_use = entry.containers.parse::<u32>().unwrap_or(0) > 0;
        Image {
            id: entry.id,
            repository: entry.repository,
            tag: entry.tag,
            size: entry.size,
            created_at: entry.created_at,
            in_use,
        }
    }
}

/// Raw JSON output from `colima status --json`
#[derive(Debug, Deserialize)]
pub struct ColimaStatusRaw {
    pub display_name: String,
    pub arch: String,
    pub runtime: String,
    pub cpu: u32,
    pub memory: u64,
    pub disk: u64,
    #[serde(default)]
    pub kubernetes: bool,
}

/// Frontend-facing colima status
#[derive(Debug, Serialize, Clone)]
pub struct ColimaStatus {
    pub running: bool,
    pub runtime: String,
    pub arch: String,
    pub cpus: u32,
    pub memory_gib: f64,
    pub disk_gib: f64,
}

impl ColimaStatusRaw {
    pub fn into_status(self) -> ColimaStatus {
        ColimaStatus {
            running: true,
            runtime: self.runtime,
            arch: self.arch,
            cpus: self.cpu,
            memory_gib: self.memory as f64 / 1_073_741_824.0,
            disk_gib: self.disk as f64 / 1_073_741_824.0,
        }
    }
}

impl ColimaStatus {
    pub fn stopped() -> Self {
        ColimaStatus {
            running: false,
            runtime: String::new(),
            arch: String::new(),
            cpus: 0,
            memory_gib: 0.0,
            disk_gib: 0.0,
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/cli/types.rs
git commit -m "feat: add Rust types for docker/colima JSON output"
```

---

### Task 4: Colima Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/colima.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Create commands module**

Create `src-tauri/src/commands/mod.rs`:

```rust
pub mod colima;
pub mod container;
pub mod image;
```

- [ ] **Step 2: Implement colima commands**

Create `src-tauri/src/commands/colima.rs`:

```rust
use crate::cli::executor::CliExecutor;
use crate::cli::types::{ColimaStatus, ColimaStatusRaw};

#[tauri::command]
pub async fn colima_status() -> Result<ColimaStatus, String> {
    let result = CliExecutor::run("colima", &["status", "--json"]).await;
    match result {
        Ok(stdout) => {
            let raw: ColimaStatusRaw = serde_json::from_str(&stdout)
                .map_err(|e| format!("Failed to parse colima status: {}", e))?;
            Ok(raw.into_status())
        }
        Err(_) => Ok(ColimaStatus::stopped()),
    }
}

#[tauri::command]
pub async fn colima_start() -> Result<(), String> {
    CliExecutor::run("colima", &["start"]).await?;
    Ok(())
}

#[tauri::command]
pub async fn colima_stop() -> Result<(), String> {
    CliExecutor::run("colima", &["stop"]).await?;
    Ok(())
}

#[tauri::command]
pub async fn colima_restart() -> Result<(), String> {
    CliExecutor::run("colima", &["stop"]).await.ok();
    CliExecutor::run("colima", &["start"]).await?;
    Ok(())
}
```

- [ ] **Step 3: Register commands in lib.rs**

Update `src-tauri/src/lib.rs` to register the colima commands. Add:

```rust
mod cli;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::colima::colima_status,
            commands::colima::colima_start,
            commands::colima::colima_stop,
            commands::colima::colima_restart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs
git commit -m "feat: add colima status/start/stop/restart commands"
```

---

### Task 5: Container Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/container.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Implement container commands**

Create `src-tauri/src/commands/container.rs`:

```rust
use crate::cli::executor::CliExecutor;
use crate::cli::types::{Container, DockerPsEntry};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[tauri::command]
pub async fn list_containers() -> Result<Vec<Container>, String> {
    let entries: Vec<DockerPsEntry> =
        CliExecutor::run_json_lines("docker", &["ps", "-a", "--format", "json"]).await?;
    Ok(entries.into_iter().map(Container::from).collect())
}

#[tauri::command]
pub async fn container_start(id: String) -> Result<(), String> {
    CliExecutor::run("docker", &["start", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn container_stop(id: String) -> Result<(), String> {
    CliExecutor::run("docker", &["stop", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn container_restart(id: String) -> Result<(), String> {
    CliExecutor::run("docker", &["restart", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn container_remove(id: String) -> Result<(), String> {
    CliExecutor::run("docker", &["rm", "-f", &id]).await?;
    Ok(())
}

#[tauri::command]
pub async fn stream_container_logs(app: AppHandle, id: String) -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let docker_host = std::env::var("DOCKER_HOST")
        .unwrap_or_else(|_| format!("unix://{}/.colima/default/docker.sock", home));

    let mut child = Command::new("docker")
        .args(["logs", "-f", "--tail", "200", &id])
        .env("DOCKER_HOST", &docker_host)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker logs: {}", e))?;

    let stdout = child.stdout.take().ok_or("No stdout")?;
    let event_name = format!("container-log-{}", id);

    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app.emit(&event_name, line);
        }
    });

    Ok(())
}
```

- [ ] **Step 2: Register container commands in lib.rs**

Add to the `invoke_handler` in `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::colima::colima_status,
    commands::colima::colima_start,
    commands::colima::colima_stop,
    commands::colima::colima_restart,
    commands::container::list_containers,
    commands::container::container_start,
    commands::container::container_stop,
    commands::container::container_restart,
    commands::container::container_remove,
    commands::container::stream_container_logs,
])
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/container.rs src-tauri/src/lib.rs
git commit -m "feat: add container list/start/stop/restart/remove/logs commands"
```

---

### Task 6: Image Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/image.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Implement image commands**

Create `src-tauri/src/commands/image.rs`:

```rust
use crate::cli::executor::CliExecutor;
use crate::cli::types::{DockerImageEntry, Image};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[tauri::command]
pub async fn list_images() -> Result<Vec<Image>, String> {
    let entries: Vec<DockerImageEntry> =
        CliExecutor::run_json_lines("docker", &["images", "--format", "json"]).await?;
    Ok(entries.into_iter().map(Image::from).collect())
}

#[tauri::command]
pub async fn pull_image(app: AppHandle, name: String) -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let docker_host = std::env::var("DOCKER_HOST")
        .unwrap_or_else(|_| format!("unix://{}/.colima/default/docker.sock", home));

    let mut child = Command::new("docker")
        .args(["pull", &name])
        .env("DOCKER_HOST", &docker_host)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker pull: {}", e))?;

    let stdout = child.stdout.take().ok_or("No stdout")?;
    let event_name = "image-pull-progress".to_string();

    let app_clone = app.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit(&event_name, line);
        }
    });

    let output = child
        .wait()
        .await
        .map_err(|e| format!("docker pull failed: {}", e))?;

    if output.success() {
        let _ = app.emit("image-pull-complete", &name);
        Ok(())
    } else {
        Err(format!("docker pull {} failed", name))
    }
}

#[tauri::command]
pub async fn remove_image(id: String) -> Result<(), String> {
    CliExecutor::run("docker", &["rmi", &id]).await?;
    Ok(())
}
```

- [ ] **Step 2: Register image commands in lib.rs**

Add to the `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::colima::colima_status,
    commands::colima::colima_start,
    commands::colima::colima_stop,
    commands::colima::colima_restart,
    commands::container::list_containers,
    commands::container::container_start,
    commands::container::container_stop,
    commands::container::container_restart,
    commands::container::container_remove,
    commands::container::stream_container_logs,
    commands::image::list_images,
    commands::image::pull_image,
    commands::image::remove_image,
])
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/image.rs src-tauri/src/lib.rs
git commit -m "feat: add image list/pull/remove commands"
```

---

### Task 7: System Tray

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs` (setup tray)

- [ ] **Step 1: Implement system tray**

Create `src-tauri/src/tray.rs`:

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

pub fn create_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit Colima Desktop", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let separator = MenuItem::with_id(app, "sep", "─────────", false, None::<&str>)?;
    let start = MenuItem::with_id(app, "colima_start", "Start Colima", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "colima_stop", "Stop Colima", true, None::<&str>)?;
    let restart =
        MenuItem::with_id(app, "colima_restart", "Restart Colima", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show, &separator, &start, &stop, &restart, &quit],
    )?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Colima Desktop")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "colima_start" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ =
                        crate::commands::colima::colima_start().await;
                    let _ = app.emit("colima-status-changed", ());
                });
            }
            "colima_stop" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ =
                        crate::commands::colima::colima_stop().await;
                    let _ = app.emit("colima-status-changed", ());
                });
            }
            "colima_restart" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ =
                        crate::commands::colima::colima_restart().await;
                    let _ = app.emit("colima-status-changed", ());
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

- [ ] **Step 2: Register tray in lib.rs**

Add tray module and setup to `src-tauri/src/lib.rs`:

```rust
mod cli;
mod commands;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::colima::colima_status,
            commands::colima::colima_start,
            commands::colima::colima_stop,
            commands::colima::colima_restart,
            commands::container::list_containers,
            commands::container::container_start,
            commands::container::container_stop,
            commands::container::container_restart,
            commands::container::container_remove,
            commands::container::stream_container_logs,
            commands::image::list_images,
            commands::image::pull_image,
            commands::image::remove_image,
        ])
        .setup(|app| {
            tray::create_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: add system tray with colima start/stop/restart"
```

---

### Task 8: TypeScript Types and Tauri Wrappers

**Files:**
- Create: `src/types/index.ts`
- Create: `src/lib/tauri.ts`

- [ ] **Step 1: Create TypeScript types**

Create `src/types/index.ts`:

```ts
export interface Container {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
  ports: string;
  created_at: string;
}

export interface Image {
  id: string;
  repository: string;
  tag: string;
  size: string;
  created_at: string;
  in_use: boolean;
}

export interface ColimaStatus {
  running: boolean;
  runtime: string;
  arch: string;
  cpus: number;
  memory_gib: number;
  disk_gib: number;
}
```

- [ ] **Step 2: Create Tauri invoke wrappers**

Create `src/lib/tauri.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { Container, Image, ColimaStatus } from "../types";

export const api = {
  // Colima
  colimaStatus: () => invoke<ColimaStatus>("colima_status"),
  colimaStart: () => invoke<void>("colima_start"),
  colimaStop: () => invoke<void>("colima_stop"),
  colimaRestart: () => invoke<void>("colima_restart"),

  // Containers
  listContainers: () => invoke<Container[]>("list_containers"),
  containerStart: (id: string) => invoke<void>("container_start", { id }),
  containerStop: (id: string) => invoke<void>("container_stop", { id }),
  containerRestart: (id: string) =>
    invoke<void>("container_restart", { id }),
  containerRemove: (id: string) => invoke<void>("container_remove", { id }),
  streamContainerLogs: (id: string) =>
    invoke<void>("stream_container_logs", { id }),

  // Images
  listImages: () => invoke<Image[]>("list_images"),
  pullImage: (name: string) => invoke<void>("pull_image", { name }),
  removeImage: (id: string) => invoke<void>("remove_image", { id }),
};
```

- [ ] **Step 3: Commit**

```bash
git add src/types/ src/lib/
git commit -m "feat: add TypeScript types and Tauri API wrappers"
```

---

### Task 9: React Query Hooks

**Files:**
- Create: `src/hooks/useColimaStatus.ts`
- Create: `src/hooks/useContainers.ts`
- Create: `src/hooks/useImages.ts`

- [ ] **Step 1: Create colima status hook**

Create `src/hooks/useColimaStatus.ts`:

```ts
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useColimaStatus() {
  return useQuery({
    queryKey: ["colima-status"],
    queryFn: api.colimaStatus,
    refetchInterval: 5000,
  });
}
```

- [ ] **Step 2: Create containers hook**

Create `src/hooks/useContainers.ts`:

```ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useContainers() {
  return useQuery({
    queryKey: ["containers"],
    queryFn: api.listContainers,
    refetchInterval: 3000,
  });
}

export function useContainerAction() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      id,
      action,
    }: {
      id: string;
      action: "start" | "stop" | "restart" | "remove";
    }) => {
      switch (action) {
        case "start":
          return api.containerStart(id);
        case "stop":
          return api.containerStop(id);
        case "restart":
          return api.containerRestart(id);
        case "remove":
          return api.containerRemove(id);
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["containers"] });
    },
  });
}
```

- [ ] **Step 3: Create images hook**

Create `src/hooks/useImages.ts`:

```ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useImages() {
  return useQuery({
    queryKey: ["images"],
    queryFn: api.listImages,
    refetchInterval: 10000,
  });
}

export function useRemoveImage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.removeImage(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["images"] });
    },
  });
}

export function usePullImage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => api.pullImage(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["images"] });
    },
  });
}
```

- [ ] **Step 4: Commit**

```bash
git add src/hooks/
git commit -m "feat: add React Query hooks for colima, containers, images"
```

---

### Task 10: Layout Components (Sidebar + MainLayout)

**Files:**
- Create: `src/components/layout/Sidebar.tsx`
- Create: `src/components/layout/MainLayout.tsx`
- Modify: `src/App.tsx`
- Modify: `src/main.tsx`

- [ ] **Step 1: Create Sidebar**

Create `src/components/layout/Sidebar.tsx`:

```tsx
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { useColimaStatus } from "../../hooks/useColimaStatus";

type Page = "containers" | "images";

interface SidebarProps {
  activePage: Page;
  onPageChange: (page: Page) => void;
}

export function Sidebar({ activePage, onPageChange }: SidebarProps) {
  const { data: status } = useColimaStatus();

  return (
    <div className="flex h-full w-52 flex-col border-r bg-muted/30 p-3">
      <div className="mb-4 flex items-center gap-2 px-2">
        <div
          className={cn(
            "h-2 w-2 rounded-full",
            status?.running ? "bg-green-500" : "bg-gray-400"
          )}
        />
        <span className="text-sm font-medium">Colima</span>
        <Badge variant={status?.running ? "default" : "secondary"} className="ml-auto text-xs">
          {status?.running ? "Running" : "Stopped"}
        </Badge>
      </div>

      <nav className="flex flex-col gap-1">
        <button
          onClick={() => onPageChange("containers")}
          className={cn(
            "rounded-md px-3 py-2 text-left text-sm transition-colors",
            activePage === "containers"
              ? "bg-accent text-accent-foreground"
              : "hover:bg-accent/50"
          )}
        >
          Containers
        </button>
        <button
          onClick={() => onPageChange("images")}
          className={cn(
            "rounded-md px-3 py-2 text-left text-sm transition-colors",
            activePage === "images"
              ? "bg-accent text-accent-foreground"
              : "hover:bg-accent/50"
          )}
        >
          Images
        </button>
      </nav>
    </div>
  );
}
```

- [ ] **Step 2: Create MainLayout**

Create `src/components/layout/MainLayout.tsx`:

```tsx
import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { ContainerList } from "../containers/ContainerList";
import { ImageList } from "../images/ImageList";

type Page = "containers" | "images";

export function MainLayout() {
  const [activePage, setActivePage] = useState<Page>("containers");

  return (
    <div className="flex h-screen">
      <Sidebar activePage={activePage} onPageChange={setActivePage} />
      <main className="flex-1 overflow-auto p-4">
        {activePage === "containers" && <ContainerList />}
        {activePage === "images" && <ImageList />}
      </main>
    </div>
  );
}
```

- [ ] **Step 3: Create placeholder ContainerList and ImageList**

These will be fully implemented in Tasks 11 and 12. Create minimal versions now so the layout compiles.

Create `src/components/containers/ContainerList.tsx`:

```tsx
export function ContainerList() {
  return <div>ContainerList placeholder</div>;
}
```

Create `src/components/images/ImageList.tsx`:

```tsx
export function ImageList() {
  return <div>ImageList placeholder</div>;
}
```

- [ ] **Step 4: Update App.tsx**

Replace `src/App.tsx`:

```tsx
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MainLayout } from "./components/layout/MainLayout";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: true,
    },
  },
});

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <MainLayout />
    </QueryClientProvider>
  );
}
```

- [ ] **Step 5: Update main.tsx if needed**

Ensure `src/main.tsx` imports the CSS correctly:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

- [ ] **Step 6: Verify it builds**

```bash
npm run tauri dev
```

Expected: Window opens with sidebar showing Colima status and two navigation items.

- [ ] **Step 7: Commit**

```bash
git add src/
git commit -m "feat: add layout with sidebar navigation and page routing"
```

---

### Task 11: Container List and Actions

**Files:**
- Modify: `src/components/containers/ContainerList.tsx`
- Create: `src/components/containers/ContainerRow.tsx`

- [ ] **Step 1: Implement ContainerRow**

Create `src/components/containers/ContainerRow.tsx`:

```tsx
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { Container } from "../../types";
import { useContainerAction } from "../../hooks/useContainers";

interface ContainerRowProps {
  container: Container;
  onViewLogs: (id: string) => void;
}

export function ContainerRow({ container, onViewLogs }: ContainerRowProps) {
  const action = useContainerAction();
  const isRunning = container.state === "running";

  return (
    <div className="flex items-center gap-3 rounded-md border px-4 py-3">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm truncate">{container.name}</span>
          <Badge variant={isRunning ? "default" : "secondary"} className="text-xs">
            {container.state}
          </Badge>
        </div>
        <div className="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
          <span className="truncate">{container.image}</span>
          {container.ports && <span>{container.ports}</span>}
        </div>
      </div>

      <div className="flex items-center gap-1">
        {isRunning ? (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => action.mutate({ id: container.id, action: "stop" })}
            disabled={action.isPending}
          >
            Stop
          </Button>
        ) : (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => action.mutate({ id: container.id, action: "start" })}
            disabled={action.isPending}
          >
            Start
          </Button>
        )}
        <Button
          variant="ghost"
          size="sm"
          onClick={() => action.mutate({ id: container.id, action: "restart" })}
          disabled={action.isPending}
        >
          Restart
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onViewLogs(container.id)}
        >
          Logs
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="text-destructive"
          onClick={() => action.mutate({ id: container.id, action: "remove" })}
          disabled={action.isPending}
        >
          Remove
        </Button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Implement ContainerList**

Replace `src/components/containers/ContainerList.tsx`:

```tsx
import { useState } from "react";
import { useContainers } from "../../hooks/useContainers";
import { ContainerRow } from "./ContainerRow";
import { ContainerLogs } from "./ContainerLogs";
import { Button } from "@/components/ui/button";

type Filter = "all" | "running" | "stopped";

export function ContainerList() {
  const { data: containers, isLoading, error } = useContainers();
  const [filter, setFilter] = useState<Filter>("all");
  const [logsContainerId, setLogsContainerId] = useState<string | null>(null);

  if (logsContainerId) {
    return (
      <ContainerLogs
        containerId={logsContainerId}
        onBack={() => setLogsContainerId(null)}
      />
    );
  }

  const filtered = containers?.filter((c) => {
    if (filter === "running") return c.state === "running";
    if (filter === "stopped") return c.state !== "running";
    return true;
  });

  return (
    <div>
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-lg font-semibold">Containers</h1>
        <div className="flex gap-1">
          {(["all", "running", "stopped"] as Filter[]).map((f) => (
            <Button
              key={f}
              variant={filter === f ? "default" : "outline"}
              size="sm"
              onClick={() => setFilter(f)}
            >
              {f.charAt(0).toUpperCase() + f.slice(1)}
            </Button>
          ))}
        </div>
      </div>

      {isLoading && <p className="text-sm text-muted-foreground">Loading...</p>}
      {error && (
        <p className="text-sm text-destructive">
          Failed to load containers. Is Colima running?
        </p>
      )}

      <div className="flex flex-col gap-2">
        {filtered?.map((container) => (
          <ContainerRow
            key={container.id}
            container={container}
            onViewLogs={setLogsContainerId}
          />
        ))}
        {filtered?.length === 0 && !isLoading && (
          <p className="text-sm text-muted-foreground">No containers found.</p>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Create ContainerLogs placeholder**

Create `src/components/containers/ContainerLogs.tsx`:

```tsx
import { Button } from "@/components/ui/button";

interface ContainerLogsProps {
  containerId: string;
  onBack: () => void;
}

export function ContainerLogs({ containerId, onBack }: ContainerLogsProps) {
  return (
    <div>
      <Button variant="ghost" size="sm" onClick={onBack} className="mb-2">
        ← Back
      </Button>
      <p>Logs for {containerId} (implemented in Task 13)</p>
    </div>
  );
}
```

- [ ] **Step 4: Verify it builds**

```bash
npm run tauri dev
```

Expected: Container list shows with filter buttons and action buttons per row.

- [ ] **Step 5: Commit**

```bash
git add src/components/containers/
git commit -m "feat: add container list with filtering and actions"
```

---

### Task 12: Image List and Management

**Files:**
- Modify: `src/components/images/ImageList.tsx`
- Create: `src/components/images/ImageRow.tsx`
- Create: `src/components/images/ImagePull.tsx`

- [ ] **Step 1: Implement ImageRow**

Create `src/components/images/ImageRow.tsx`:

```tsx
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { Image } from "../../types";
import { useRemoveImage } from "../../hooks/useImages";

interface ImageRowProps {
  image: Image;
}

export function ImageRow({ image }: ImageRowProps) {
  const remove = useRemoveImage();

  return (
    <div className="flex items-center gap-3 rounded-md border px-4 py-3">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm truncate">
            {image.repository}:{image.tag}
          </span>
          {image.in_use && (
            <Badge variant="outline" className="text-xs">
              In use
            </Badge>
          )}
        </div>
        <div className="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
          <span>{image.size}</span>
          <span>{image.id.slice(0, 12)}</span>
        </div>
      </div>

      <Button
        variant="ghost"
        size="sm"
        className="text-destructive"
        onClick={() => remove.mutate(image.id)}
        disabled={remove.isPending || image.in_use}
      >
        Remove
      </Button>
    </div>
  );
}
```

- [ ] **Step 2: Implement ImagePull**

Create `src/components/images/ImagePull.tsx`:

```tsx
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { usePullImage } from "../../hooks/useImages";

export function ImagePull() {
  const [name, setName] = useState("");
  const pull = usePullImage();

  const handlePull = () => {
    if (!name.trim()) return;
    pull.mutate(name.trim());
    setName("");
  };

  return (
    <div className="flex gap-2">
      <Input
        placeholder="Image name (e.g. nginx:alpine)"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && handlePull()}
        className="max-w-sm"
      />
      <Button onClick={handlePull} disabled={pull.isPending || !name.trim()}>
        {pull.isPending ? "Pulling..." : "Pull"}
      </Button>
    </div>
  );
}
```

- [ ] **Step 3: Implement ImageList**

Replace `src/components/images/ImageList.tsx`:

```tsx
import { useImages } from "../../hooks/useImages";
import { ImageRow } from "./ImageRow";
import { ImagePull } from "./ImagePull";

export function ImageList() {
  const { data: images, isLoading, error } = useImages();

  const totalSize = images
    ?.map((img) => {
      const match = img.size.match(/([\d.]+)(GB|MB|KB)/);
      if (!match) return 0;
      const val = parseFloat(match[1]);
      if (match[2] === "GB") return val;
      if (match[2] === "MB") return val / 1024;
      return val / 1_048_576;
    })
    .reduce((sum, v) => sum + v, 0);

  return (
    <div>
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold">Images</h1>
          {totalSize !== undefined && (
            <p className="text-xs text-muted-foreground">
              Total: {totalSize.toFixed(2)} GB ({images?.length} images)
            </p>
          )}
        </div>
      </div>

      <div className="mb-4">
        <ImagePull />
      </div>

      {isLoading && <p className="text-sm text-muted-foreground">Loading...</p>}
      {error && (
        <p className="text-sm text-destructive">
          Failed to load images. Is Colima running?
        </p>
      )}

      <div className="flex flex-col gap-2">
        {images?.map((image) => (
          <ImageRow key={image.id} image={image} />
        ))}
        {images?.length === 0 && !isLoading && (
          <p className="text-sm text-muted-foreground">No images found.</p>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Verify it builds**

```bash
npm run tauri dev
```

Expected: Images page shows list with pull input, size summary, and remove buttons.

- [ ] **Step 5: Commit**

```bash
git add src/components/images/
git commit -m "feat: add image list with pull and remove functionality"
```

---

### Task 13: Container Log Streaming

**Files:**
- Modify: `src/components/containers/ContainerLogs.tsx`

- [ ] **Step 1: Implement log streaming component**

Replace `src/components/containers/ContainerLogs.tsx`:

```tsx
import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { api } from "../../lib/tauri";

interface ContainerLogsProps {
  containerId: string;
  onBack: () => void;
}

export function ContainerLogs({ containerId, onBack }: ContainerLogsProps) {
  const [logs, setLogs] = useState<string[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    api.streamContainerLogs(containerId);

    const unlisten = listen<string>(
      `container-log-${containerId}`,
      (event) => {
        setLogs((prev) => [...prev, event.payload]);
      }
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [containerId]);

  useEffect(() => {
    if (autoScroll) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, autoScroll]);

  return (
    <div className="flex h-full flex-col">
      <div className="mb-2 flex items-center gap-2">
        <Button variant="ghost" size="sm" onClick={onBack}>
          ← Back
        </Button>
        <span className="text-sm font-medium">
          Logs: {containerId.slice(0, 12)}
        </span>
        <Button
          variant="outline"
          size="sm"
          className="ml-auto"
          onClick={() => setAutoScroll(!autoScroll)}
        >
          {autoScroll ? "Auto-scroll: On" : "Auto-scroll: Off"}
        </Button>
      </div>

      <ScrollArea className="flex-1 rounded-md border bg-black p-3">
        <pre className="text-xs text-green-400 font-mono whitespace-pre-wrap">
          {logs.join("\n")}
        </pre>
        <div ref={bottomRef} />
      </ScrollArea>
    </div>
  );
}
```

- [ ] **Step 2: Verify it builds**

```bash
npm run tauri dev
```

Expected: Clicking "Logs" on a running container shows streaming log output.

- [ ] **Step 3: Commit**

```bash
git add src/components/containers/ContainerLogs.tsx
git commit -m "feat: add real-time container log streaming"
```

---

### Task 14: Window Close → Hide to Tray Behavior

**Files:**
- Modify: `src-tauri/src/lib.rs` (add on_window_event)

- [ ] **Step 1: Add close-to-tray behavior**

In `src-tauri/src/lib.rs`, add a window event handler so that closing the main window hides it instead of quitting:

```rust
mod cli;
mod commands;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::colima::colima_status,
            commands::colima::colima_start,
            commands::colima::colima_stop,
            commands::colima::colima_restart,
            commands::container::list_containers,
            commands::container::container_start,
            commands::container::container_stop,
            commands::container::container_restart,
            commands::container::container_remove,
            commands::container::stream_container_logs,
            commands::image::list_images,
            commands::image::pull_image,
            commands::image::remove_image,
        ])
        .setup(|app| {
            tray::create_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify behavior**

```bash
npm run tauri dev
```

Expected: Clicking the window close button hides the window. The app remains in the system tray. Clicking "Show Window" from the tray menu brings it back.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: hide window to system tray on close instead of quitting"
```

---

### Task 15: Final Integration Test and DMG Build

- [ ] **Step 1: Run the full app in dev mode**

```bash
npm run tauri dev
```

Manual verification checklist:
- System tray icon appears
- Tray menu shows Start/Stop/Restart/Show Window/Quit
- Main window shows sidebar with Colima status
- Containers page lists running containers
- Container actions (start/stop/restart/remove) work
- Container logs stream in real-time
- Images page lists images with sizes
- Image pull works
- Image remove works
- Closing window hides to tray
- "Show Window" from tray restores the window

- [ ] **Step 2: Build DMG**

```bash
npm run tauri build
```

Expected: DMG file is created at `src-tauri/target/release/bundle/dmg/Colima Desktop_0.1.0_aarch64.dmg`

- [ ] **Step 3: Commit any final fixes**

```bash
git add -A
git commit -m "chore: finalize MVP build configuration"
```
