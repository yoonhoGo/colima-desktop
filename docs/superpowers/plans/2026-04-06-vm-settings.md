# VM Resource Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Colima VM의 CPU, Memory, Disk, Runtime, Network Address 설정을 UI에서 조정하고 즉시 적용(restart)하는 기능 추가

**Architecture:** Backend(Rust)에서 `colima list --json`으로 현재 설정 조회, `sysctl`로 호스트 정보 조회, `colima stop/start`로 설정 적용. Frontend(React)에서 슬라이더+숫자 입력 폼으로 설정 편집, React Query mutation으로 적용.

**Tech Stack:** Tauri 2 (Rust), React 19, TanStack React Query, Tailwind CSS, shadcn/ui, Lucide Icons

---

## File Structure

### New files
| File | Responsibility |
|------|---------------|
| `src-tauri/src/commands/vm_settings.rs` | Tauri commands: get_vm_settings, get_host_info, apply_vm_settings |
| `src/components/settings/VmSettings.tsx` | Settings page main component |
| `src/hooks/useVmSettings.ts` | React Query hooks for VM settings |

### Modified files
| File | Changes |
|------|---------|
| `src-tauri/src/cli/types.rs` | Add VmSettings, HostInfo, ColimaListEntry types |
| `src-tauri/src/commands/mod.rs` | Register vm_settings module |
| `src-tauri/src/lib.rs` | Register new Tauri commands |
| `src/types/index.ts` | Add VmSettings, HostInfo types |
| `src/lib/tauri.ts` | Add API functions |
| `src/components/layout/Sidebar.tsx` | Add "Settings" nav item |
| `src/components/layout/MainLayout.tsx` | Add settings page routing |

---

### Task 1: Backend Types (Rust)

**Files:**
- Modify: `src-tauri/src/cli/types.rs`

- [ ] **Step 1: Add VmSettings and HostInfo types to types.rs**

Append to end of `src-tauri/src/cli/types.rs`:

```rust
#[derive(Debug, Serialize, Clone)]
pub struct VmSettings {
    pub cpus: u32,
    pub memory_gib: f64,
    pub disk_gib: f64,
    pub runtime: String,
    pub network_address: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct HostInfo {
    pub cpus: u32,
    pub memory_gib: f64,
}

#[derive(Debug, Deserialize)]
pub struct ColimaListEntry {
    pub cpus: u32,
    pub memory: u64,
    pub disk: u64,
    pub runtime: String,
    #[serde(default)]
    pub network_address: String,
    pub status: String,
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors (warnings ok)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/cli/types.rs
git commit -m "feat(backend): add VmSettings, HostInfo, ColimaListEntry types"
```

---

### Task 2: Backend Commands (Rust)

**Files:**
- Create: `src-tauri/src/commands/vm_settings.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create vm_settings.rs with three Tauri commands**

Create `src-tauri/src/commands/vm_settings.rs`:

```rust
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
```

- [ ] **Step 2: Register the module in mod.rs**

Add to `src-tauri/src/commands/mod.rs`:

```rust
pub mod vm_settings;
```

- [ ] **Step 3: Register commands in lib.rs**

Add to the `generate_handler!` macro in `src-tauri/src/lib.rs`, after the image commands:

```rust
commands::vm_settings::get_vm_settings,
commands::vm_settings::get_host_info,
commands::vm_settings::apply_vm_settings,
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/vm_settings.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(backend): add get_vm_settings, get_host_info, apply_vm_settings commands"
```

---

### Task 3: Frontend Types & API Layer

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: Add TypeScript types**

Append to `src/types/index.ts`:

```typescript
export interface VmSettings {
  cpus: number;
  memory_gib: number;
  disk_gib: number;
  runtime: string;
  network_address: string;
}

export interface HostInfo {
  cpus: number;
  memory_gib: number;
}
```

- [ ] **Step 2: Add API functions to tauri.ts**

Update import in `src/lib/tauri.ts`:

```typescript
import type { Container, Image, ColimaStatus, VmSettings, HostInfo } from "../types";
```

Add to the `api` object:

```typescript
getVmSettings: () => invoke<VmSettings>("get_vm_settings"),
getHostInfo: () => invoke<HostInfo>("get_host_info"),
applyVmSettings: (settings: { cpus: number; memoryGib: number; diskGib: number; runtime: string; networkAddress: string }) =>
  invoke<void>("apply_vm_settings", settings),
```

- [ ] **Step 3: Commit**

```bash
git add src/types/index.ts src/lib/tauri.ts
git commit -m "feat(frontend): add VM settings types and API functions"
```

---

### Task 4: React Query Hook

**Files:**
- Create: `src/hooks/useVmSettings.ts`

- [ ] **Step 1: Create useVmSettings hook**

Create `src/hooks/useVmSettings.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useVmSettings() {
  return useQuery({
    queryKey: ["vm-settings"],
    queryFn: api.getVmSettings,
    refetchInterval: 10000,
  });
}

export function useHostInfo() {
  return useQuery({
    queryKey: ["host-info"],
    queryFn: api.getHostInfo,
    staleTime: Infinity,
  });
}

export function useApplyVmSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: {
      cpus: number;
      memoryGib: number;
      diskGib: number;
      runtime: string;
      networkAddress: string;
    }) => api.applyVmSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["vm-settings"] });
      queryClient.invalidateQueries({ queryKey: ["colima-status"] });
    },
  });
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/useVmSettings.ts
git commit -m "feat(frontend): add useVmSettings, useHostInfo, useApplyVmSettings hooks"
```

---

### Task 5: Settings Page Component

**Files:**
- Create: `src/components/settings/VmSettings.tsx`

- [ ] **Step 1: Create VmSettings page component**

Create `src/components/settings/VmSettings.tsx`:

```tsx
import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Loader2 } from "lucide-react";
import { useVmSettings, useHostInfo, useApplyVmSettings } from "@/hooks/useVmSettings";
import { useColimaStatus } from "@/hooks/useColimaStatus";

export function VmSettings() {
  const { data: settings, isLoading: settingsLoading, error: settingsError } = useVmSettings();
  const { data: hostInfo } = useHostInfo();
  const { data: status } = useColimaStatus();
  const applyMutation = useApplyVmSettings();

  const [cpus, setCpus] = useState(2);
  const [memoryGib, setMemoryGib] = useState(2);
  const [diskGib, setDiskGib] = useState(60);
  const [runtime, setRuntime] = useState("docker");
  const [networkAddress, setNetworkAddress] = useState("");

  useEffect(() => {
    if (settings) {
      setCpus(settings.cpus);
      setMemoryGib(Math.round(settings.memory_gib));
      setDiskGib(Math.round(settings.disk_gib));
      setRuntime(settings.runtime);
      setNetworkAddress(settings.network_address);
    }
  }, [settings]);

  const hasChanges =
    settings &&
    (cpus !== settings.cpus ||
      memoryGib !== Math.round(settings.memory_gib) ||
      diskGib !== Math.round(settings.disk_gib) ||
      runtime !== settings.runtime ||
      networkAddress !== settings.network_address);

  const currentDiskGib = settings ? Math.round(settings.disk_gib) : 0;
  const diskShrinkWarning = settings && diskGib < currentDiskGib;

  const handleApply = () => {
    applyMutation.mutate({
      cpus,
      memoryGib,
      diskGib,
      runtime,
      networkAddress,
    });
  };

  const maxCpus = hostInfo?.cpus ?? 16;
  const maxMemory = hostInfo ? Math.floor(hostInfo.memory_gib) : 64;

  if (settingsLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (settingsError) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Failed to load VM settings. Is Colima running?
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">VM Settings</h2>
        <Badge variant={status?.running ? "default" : "secondary"}>
          {status?.running ? "Running" : "Stopped"}
        </Badge>
      </div>

      <div className="space-y-5">
        {/* CPU */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium">CPU</label>
            <span className="text-xs text-muted-foreground">max {maxCpus}</span>
          </div>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min={1}
              max={maxCpus}
              value={cpus}
              onChange={(e) => setCpus(Number(e.target.value))}
              disabled={applyMutation.isPending}
              className="flex-1"
            />
            <Input
              type="number"
              min={1}
              max={maxCpus}
              value={cpus}
              onChange={(e) => setCpus(Math.min(maxCpus, Math.max(1, Number(e.target.value))))}
              disabled={applyMutation.isPending}
              className="w-20 text-center"
            />
          </div>
        </div>

        {/* Memory */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium">Memory (GiB)</label>
            <span className="text-xs text-muted-foreground">max {maxMemory}</span>
          </div>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min={1}
              max={maxMemory}
              value={memoryGib}
              onChange={(e) => setMemoryGib(Number(e.target.value))}
              disabled={applyMutation.isPending}
              className="flex-1"
            />
            <Input
              type="number"
              min={1}
              max={maxMemory}
              value={memoryGib}
              onChange={(e) => setMemoryGib(Math.min(maxMemory, Math.max(1, Number(e.target.value))))}
              disabled={applyMutation.isPending}
              className="w-20 text-center"
            />
          </div>
        </div>

        {/* Disk */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium">Disk (GiB)</label>
            <span className="text-xs text-muted-foreground">10 ~ 500</span>
          </div>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min={10}
              max={500}
              value={diskGib}
              onChange={(e) => setDiskGib(Number(e.target.value))}
              disabled={applyMutation.isPending}
              className="flex-1"
            />
            <Input
              type="number"
              min={10}
              max={500}
              value={diskGib}
              onChange={(e) => setDiskGib(Math.min(500, Math.max(10, Number(e.target.value))))}
              disabled={applyMutation.isPending}
              className="w-20 text-center"
            />
          </div>
          {diskShrinkWarning && (
            <p className="text-xs text-destructive">
              Disk cannot be shrunk below current size ({currentDiskGib} GiB). This value will be ignored.
            </p>
          )}
        </div>

        {/* Runtime */}
        <div className="space-y-2">
          <label className="text-sm font-medium">Runtime</label>
          <div className="flex gap-2">
            {(["docker", "containerd"] as const).map((r) => (
              <Button
                key={r}
                variant={runtime === r ? "default" : "outline"}
                size="sm"
                onClick={() => setRuntime(r)}
                disabled={applyMutation.isPending}
              >
                {r}
              </Button>
            ))}
          </div>
        </div>

        {/* Network Address */}
        <div className="space-y-2">
          <label className="text-sm font-medium">Network Address</label>
          <Input
            placeholder="e.g. 192.168.106.2 (optional)"
            value={networkAddress}
            onChange={(e) => setNetworkAddress(e.target.value)}
            disabled={applyMutation.isPending}
          />
        </div>
      </div>

      {/* Apply Button */}
      <div className="space-y-2">
        <Button
          onClick={handleApply}
          disabled={!hasChanges || applyMutation.isPending}
          className="w-full"
        >
          {applyMutation.isPending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Restarting...
            </>
          ) : (
            "Save & Restart"
          )}
        </Button>

        {applyMutation.isError && (
          <p className="text-center text-xs text-destructive">
            {applyMutation.error?.message ?? "Failed to apply settings"}
          </p>
        )}

        {applyMutation.isSuccess && (
          <p className="text-center text-xs text-green-600">
            Settings applied successfully
          </p>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/settings/VmSettings.tsx
git commit -m "feat(frontend): add VmSettings page component"
```

---

### Task 6: Sidebar & Routing Integration

**Files:**
- Modify: `src/components/layout/Sidebar.tsx`
- Modify: `src/components/layout/MainLayout.tsx`

- [ ] **Step 1: Update Sidebar to add Settings menu**

In `src/components/layout/Sidebar.tsx`:

1. Update the `Page` type:
```typescript
type Page = "containers" | "images" | "settings";
```

2. Add the Settings button after the Images button inside `<nav>`:
```tsx
<button
  onClick={() => onPageChange("settings")}
  className={cn("rounded-md px-3 py-2 text-left text-sm transition-colors",
    activePage === "settings" ? "bg-accent text-accent-foreground" : "hover:bg-accent/50"
  )}
>
  Settings
</button>
```

- [ ] **Step 2: Update MainLayout to route to Settings page**

In `src/components/layout/MainLayout.tsx`:

1. Add import:
```typescript
import { VmSettings } from "../settings/VmSettings";
```

2. Update the `Page` type:
```typescript
type Page = "containers" | "images" | "settings";
```

3. Add the settings route inside `<main>`, after the images conditional:
```tsx
{activePage === "settings" && <VmSettings />}
```

- [ ] **Step 3: Verify the app builds**

Run: `npm run build`
Expected: Build succeeds with no TypeScript errors

- [ ] **Step 4: Commit**

```bash
git add src/components/layout/Sidebar.tsx src/components/layout/MainLayout.tsx
git commit -m "feat: integrate settings page into sidebar and routing"
```

---

### Task 7: Build & Smoke Test

- [ ] **Step 1: Run full Tauri build check**

Run: `cd src-tauri && cargo check`
Expected: No errors

- [ ] **Step 2: Run frontend type check**

Run: `npx tsc --noEmit`
Expected: No type errors

- [ ] **Step 3: Run dev server to verify integration**

Run: `npm run tauri dev`
Expected: App launches, "Settings" appears in sidebar, clicking it shows the VM settings form

- [ ] **Step 4: Final commit if any fixes needed**

If fixes were needed during smoke test, commit them:
```bash
git add -A
git commit -m "fix: address issues found during VM settings smoke test"
```
