# mDNS Service Publishing Design

## Overview

Colima Desktop에서 실행 중인 Docker 컨테이너를 LAN에 mDNS 서비스로 자동 노출하는 기능.
같은 네트워크의 다른 기기에서 `container-name.local`로 컨테이너에 접근할 수 있게 한다.

## Requirements

- **서비스 등록(Publishing)** 전용 — 서비스 검색(Browsing)은 범위 밖
- **자동 등록 + 수동 오버라이드**: 실행 중인 컨테이너 자동 등록, 컨테이너별 커스터마이징 가능
- **사용자 정의 hostname**: 기본값은 컨테이너 이름, 사용자가 원하는 이름으로 오버라이드 가능
- **폴링 기반 동기화**: 5초 주기로 `docker ps` 실행하여 컨테이너 상태와 mDNS 등록 동기화
- **설정 영속화**: Tauri 앱 데이터 경로에 JSON 파일로 저장
- **UI**: Containers 탭에 인라인 표시 + Settings에 글로벌 설정

## Approach

Rust `mdns-sd` 크레이트를 사용하여 Tauri 프로세스 내에서 mDNS 데몬을 직접 운영한다.

선택 이유:
- 순수 Rust, 외부 시스템 의존성 없음 (Avahi/Bonjour 불필요)
- cross-platform (macOS, Linux, Windows)
- 기존 worktree에서 검증된 접근방식
- Tauri State 패턴으로 라이프사이클 관리가 깔끔

## Backend Architecture (Rust/Tauri)

### Module Structure

```
src-tauri/src/
├── mdns/
│   ├── mod.rs          # 모듈 공개 인터페이스
│   ├── manager.rs      # MdnsManager - 서비스 등록/해제 핵심 로직
│   ├── config.rs       # MdnsConfig - 설정 영속화 (JSON)
│   └── sync.rs         # 컨테이너 ↔ mDNS 동기화 로직
├── commands/
│   └── mdns.rs         # Tauri IPC 커맨드 핸들러
```

### Core Types

```rust
// 글로벌 설정
struct MdnsConfig {
    enabled: bool,                          // mDNS 전체 활성화
    auto_register: bool,                    // 자동 등록 여부
    default_service_type: String,           // 기본 "_http._tcp.local."
    container_overrides: HashMap<String, ContainerMdnsConfig>,
}

// 컨테이너별 오버라이드
struct ContainerMdnsConfig {
    enabled: bool,                          // 이 컨테이너 등록 여부
    hostname: Option<String>,               // 커스텀 hostname (None이면 컨테이너 이름)
    service_type: Option<String>,           // 커스텀 서비스 타입
    port: Option<u16>,                      // 커스텀 포트 (None이면 자동 감지)
}
```

### MdnsManager

- `Mutex<Option<ServiceDaemon>>`으로 Tauri `State`에 등록
- `enabled=true`일 때만 `ServiceDaemon` 생성
- 등록된 서비스 목록을 인메모리 `HashMap<String, ServiceInfo>`로 추적
- 앱 종료 시 `ServiceDaemon::shutdown()`으로 정리

### Tauri Commands

| Command | Description |
|---------|-------------|
| `mdns_get_config` | 현재 설정 조회 |
| `mdns_set_config` | 글로벌 설정 변경 (enabled, auto_register 등) |
| `mdns_set_container_override` | 컨테이너별 오버라이드 설정 |
| `mdns_remove_container_override` | 컨테이너 오버라이드 제거 |
| `mdns_sync_containers` | 동기화 실행 + 현재 등록 상태 반환 |
| `mdns_get_status` | mDNS 데몬 상태 및 등록된 서비스 목록 조회 |

## Frontend Architecture (React/TypeScript)

### Types

```typescript
interface MdnsConfig {
  enabled: boolean;
  autoRegister: boolean;
  defaultServiceType: string;
  containerOverrides: Record<string, ContainerMdnsConfig>;
}

interface ContainerMdnsConfig {
  enabled: boolean;
  hostname?: string;
  serviceType?: string;
  port?: number;
}

interface MdnsServiceEntry {
  containerId: string;
  containerName: string;
  hostname: string;
  serviceType: string;
  port: number;
  registered: boolean;
}

interface MdnsSyncResult {
  services: MdnsServiceEntry[];
  daemonRunning: boolean;
}
```

### React Query Hooks

```typescript
useMdnsConfig()         // mdns_get_config 조회 + 캐싱
useMdnsSync()           // mdns_sync_containers, 5초 폴링 (enabled일 때만)
useMdnsSetConfig()      // mdns_set_config mutation
useMdnsSetOverride()    // mdns_set_container_override mutation
useMdnsRemoveOverride() // mdns_remove_container_override mutation
```

### UI Layout

**Containers Tab — Inline Display**

각 컨테이너 행에 mDNS 상태를 표시. mDNS 전역 비활성화 시 숨김.
컨테이너별 `[Configure]` 버튼으로 오버라이드 다이얼로그 열기.

**Settings — Global mDNS Settings**

기존 Settings 영역에 "mDNS" 섹션 추가:
- Enable mDNS 토글
- Auto-register 토글
- Default Service Type 선택
- 등록된 서비스 수 표시

### Components

```
src/components/
├── containers/
│   └── ContainerMdnsBadge.tsx    # 컨테이너 행 내 mDNS 상태 배지
│   └── ContainerMdnsDialog.tsx   # 컨테이너별 오버라이드 설정 다이얼로그
├── settings/
│   └── MdnsSettings.tsx          # 글로벌 mDNS 설정 패널
```

## Data Flow

### Sync Cycle (5-second polling)

```
Frontend: useMdnsSync() → invoke("mdns_sync_containers")
  ↓
Backend:
  1. docker ps → 실행 중인 컨테이너 + 노출 포트 목록
  2. MdnsConfig.containerOverrides 적용
     - override.enabled=false → 건너뜀
     - override.hostname → 기본 컨테이너 이름 대체
  3. 현재 등록 목록과 diff
     - 새 컨테이너 → ServiceDaemon.register()
     - 종료된 컨테이너 → ServiceDaemon.unregister()
  4. MdnsSyncResult 반환
  ↓
Frontend: React Query 캐시 업데이트 → UI 렌더링
```

### Config Change Flow

```
Frontend mutation → mdns_set_config
  → MdnsConfig 저장 (JSON 파일)
  → enabled 변경 시: ServiceDaemon 생성/삭제
  → 즉시 mdns_sync_containers 실행
  → invalidateQueries로 UI 갱신
```

## Error Handling

| Scenario | Handling |
|----------|----------|
| Colima 미실행 | `docker ps` 실패 → 동기화 건너뜀, 기존 등록 유지, 프론트엔드에 상태 전달 |
| 포트 미노출 컨테이너 | auto_register 대상에서 제외 (수동 오버라이드로만 등록 가능) |
| 멀티캐스트 소켓 바인딩 실패 | ServiceDaemon 생성 실패 → 에러 로그 + `daemonRunning: false` 반환 |
| 서비스 이름 충돌 | `mdns-sd`가 자동으로 이름 뒤에 숫자 추가 (RFC 6763 표준 동작) |
| 설정 파일 손상 | 기본값으로 폴백, 다음 저장 시 정상 파일로 덮어씀 |

## App Lifecycle

| Event | Action |
|-------|--------|
| 앱 시작 | Config 로드 → 조건부 데몬 시작 |
| Colima 시작/정지 | 다음 폴링 주기에 자동 반영 |
| 앱 종료 | ServiceDaemon::shutdown() → 모든 서비스 해제 |
| 앱 크래시 | OS가 소켓 정리, mDNS 레코드 TTL 만료 후 자동 소멸 |

## Dependencies

### Rust (Cargo.toml)

- `mdns-sd` — mDNS service registration (async feature)
- `local-ip-address` — 호스트 LAN IP 감지
- `hostname` — 호스트 이름 감지

### Frontend (package.json)

- 추가 의존성 없음 (기존 React Query, Tauri API, shadcn/ui 활용)
