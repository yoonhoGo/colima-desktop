# VM Resource Settings Feature Design

## Overview

Colima Desktop에 VM 자원 설정 페이지를 추가하여 CPU, Memory, Disk, Runtime, Network Address를 조정할 수 있게 한다. 설정 변경 시 VM을 자동으로 restart하여 즉시 적용한다.

## Settings Items

| Item | Type | Constraints |
|------|------|-------------|
| CPU | Slider + Number input | 1 ~ host logical cores |
| Memory (GiB) | Slider + Number input | 1 ~ host physical memory |
| Disk (GiB) | Slider + Number input | 10 ~ 500 |
| Runtime | Radio / Segment control | docker / containerd |
| Network Address | Text input | Optional, empty = default |

## Architecture

### Backend (Rust / Tauri)

**New Tauri commands:**

1. `get_vm_settings` — 현재 VM 설정값 조회
   - `colima list --json`을 파싱하여 CPU, Memory, Disk, Runtime 반환
   - 호스트 시스템 정보(코어 수, 메모리)도 함께 반환하여 슬라이더 max 값에 사용

2. `apply_vm_settings` — 설정 적용
   - `colima stop` → `colima start --cpu X --memory Y --disk Z --runtime R [--network-address A]`
   - 실행 중이 아닌 경우 `colima start`만 실행
   - 결과(성공/실패) 반환

3. `get_host_info` — 호스트 시스템 정보 조회
   - macOS: `sysctl -n hw.ncpu` (CPU), `sysctl -n hw.memsize` (Memory)
   - 슬라이더 max 값 결정에 사용

**New types:**

```rust
pub struct VmSettings {
    pub cpus: u32,
    pub memory_gib: f64,
    pub disk_gib: f64,
    pub runtime: String,
    pub network_address: Option<String>,
}

pub struct HostInfo {
    pub cpus: u32,
    pub memory_gib: f64,
}
```

### Frontend (React / TypeScript)

**New components:**

1. `src/components/settings/VmSettings.tsx` — 설정 페이지 메인 컴포넌트
   - 상단: "VM Settings" 제목 + VM 상태 배지 (Running/Stopped)
   - 중단: 각 설정 항목 (라벨 + 슬라이더 + 숫자 입력)
   - 하단: "Save & Restart" 버튼 (변경사항 있을 때만 활성화)
   - Restart 중 버튼 비활성화 + 스피너

2. `src/components/settings/SettingSlider.tsx` — 재사용 슬라이더 + 숫자 입력 컴포넌트

**New hooks:**

1. `src/hooks/useVmSettings.ts`
   - `useQuery` — 현재 VM 설정 + 호스트 정보 조회 (refetch interval: 10s)
   - `useMutation` — 설정 적용 (성공 시 colima-status, vm-settings 쿼리 무효화)

**New types:**

```typescript
interface VmSettings {
  cpus: number;
  memoryGib: number;
  diskGib: number;
  runtime: 'docker' | 'containerd';
  networkAddress: string | null;
}

interface HostInfo {
  cpus: number;
  memoryGib: number;
}
```

**Sidebar update:**

- `Sidebar.tsx`에 "Settings" 메뉴 항목 추가 (Containers, Images 아래)
- 아이콘: `lucide-react`의 `Settings` 아이콘

**API layer update:**

- `src/lib/tauri.ts`에 `getVmSettings()`, `applyVmSettings()`, `getHostInfo()` 함수 추가

## Data Flow

```
Settings Page → Save & Restart 클릭
  → api.applyVmSettings({ cpus, memoryGib, diskGib, runtime, networkAddress })
  → Tauri invoke("apply_vm_settings")
  → Rust: colima stop (if running) → colima start --cpu --memory --disk --runtime [--network-address]
  → Result<(), String> 반환
  → 성공: 쿼리 무효화 → UI 갱신
  → 실패: 에러 메시지 표시
```

## UI Behavior

- **초기 로드**: 현재 VM 설정값을 조회하여 폼에 채움. VM이 꺼져있으면 기본값 표시.
- **변경 감지**: 현재 값과 폼 값을 비교하여 변경사항이 있을 때만 버튼 활성화
- **적용 중**: 버튼을 "Restarting..." 텍스트 + 스피너로 교체, 모든 입력 비활성화
- **에러 처리**: 실패 시 에러 메시지를 인라인으로 표시
- **Disk 주의사항**: Disk은 축소 불가능 — 현재 값보다 작게 설정 시 경고 표시

## File Changes Summary

### New files
- `src/components/settings/VmSettings.tsx`
- `src/components/settings/SettingSlider.tsx`
- `src/hooks/useVmSettings.ts`
- `src-tauri/src/commands/vm_settings.rs`

### Modified files
- `src/lib/tauri.ts` — API 함수 추가
- `src/types/index.ts` — 타입 추가
- `src/components/layout/Sidebar.tsx` — Settings 메뉴 추가
- `src/App.tsx` — Settings 페이지 라우팅 추가
- `src-tauri/src/commands/mod.rs` — vm_settings 모듈 등록
- `src-tauri/src/lib.rs` — Tauri 커맨드 등록
