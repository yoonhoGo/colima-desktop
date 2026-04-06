# Docker Compose Container Grouping Design

## Overview

컨테이너 목록에서 Docker Compose로 실행된 컨테이너들을 프로젝트 단위로 묶어 아코디언(접기/펼치기) UI로 표시한다. 그룹 단위 액션(Start All/Stop All/Restart All/Remove All)과 개별 컨테이너 액션을 모두 지원한다.

## Data Collection

현재 `docker ps -a --format json`은 Compose 라벨을 포함하지 않는다. format 문자열에 Labels 필드를 추가하여 Compose 관련 라벨을 수집한다.

**필요한 Docker 라벨:**
- `com.docker.compose.project` — Compose 프로젝트명 (그룹핑 키)
- `com.docker.compose.service` — 서비스명 (개별 컨테이너 식별)

**수집 방식:**
- `docker ps -a --format json` 출력에 이미 Labels 필드가 포함됨 (JSON format일 때)
- Rust에서 Labels 필드를 파싱하여 compose 관련 라벨 추출

## Backend Changes

### Type Changes (`src-tauri/src/cli/types.rs`)

Container 타입에 compose 관련 필드 추가:

```rust
pub struct Container {
    // ... existing fields ...
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,
}
```

DockerPsEntry에 Labels 필드 추가:

```rust
pub struct DockerPsEntry {
    // ... existing fields ...
    pub labels: String,
}
```

Labels 문자열에서 compose 라벨을 파싱하는 로직을 `From<DockerPsEntry> for Container` 구현에 추가.

### Command Changes

`list_containers` 커맨드는 변경 불필요 — DockerPsEntry 타입 변경으로 자동 반영.

그룹 액션은 **별도 backend 커맨드 불필요** — frontend에서 기존 개별 컨테이너 커맨드를 순차 호출.

## Frontend Changes

### Type Changes (`src/types/index.ts`)

```typescript
export interface Container {
  // ... existing fields ...
  compose_project: string | null;
  compose_service: string | null;
}
```

### New Component: ComposeGroup (`src/components/containers/ComposeGroup.tsx`)

- 아코디언 헤더: 프로젝트명 + 상태 요약 (N/M running) + 그룹 액션 버튼
- 접기/펼치기 토글 (기본: 접힘)
- 펼치면: 개별 컨테이너 행 (service명 우선 표시) + 개별 액션

### Modified Component: ContainerList (`src/components/containers/ContainerList.tsx`)

그룹핑 로직 추가:
1. `compose_project`가 있는 컨테이너를 프로젝트명으로 그룹핑
2. compose 그룹 → ComposeGroup 컴포넌트로 렌더링
3. 단독 컨테이너(compose_project === null) → 기존 ContainerRow로 렌더링
4. 필터(All/Running/Stopped)는 개별 컨테이너 단위로 적용 — 그룹 내 모든 컨테이너가 필터에 걸리면 그룹 자체를 숨김

### Group Actions

그룹 단위 액션은 frontend에서 그룹 내 컨테이너들의 개별 mutation을 순차 호출:
- **Start All** — state !== "running"인 컨테이너만 start
- **Stop All** — state === "running"인 컨테이너만 stop
- **Restart All** — 모든 컨테이너 restart
- **Remove All** — 모든 컨테이너 rm -f

### UI Layout

```
[필터: All | Running | Stopped]

▶ my-app (2/3 running)           [Start] [Stop] [Restart] [Remove]

▼ backend-stack (3/3 running)    [Start] [Stop] [Restart] [Remove]
  ├─ api        running  nginx:latest     0.0.0.0:8080→80
  ├─ db         running  postgres:15      0.0.0.0:5432→5432
  └─ redis      running  redis:7          0.0.0.0:6379→6379

standalone-container  running  ubuntu:22.04  [Start] [Stop] [Restart] [Logs] [Remove]
```

- Compose 그룹이 먼저, 단독 컨테이너가 아래에 표시
- 그룹 헤더에 chevron 아이콘(▶/▼)으로 접힘/펼침 표시

## File Changes Summary

### New files
- `src/components/containers/ComposeGroup.tsx` — Compose 그룹 아코디언 컴포넌트

### Modified files
- `src-tauri/src/cli/types.rs` — DockerPsEntry에 labels, Container에 compose 필드 추가
- `src/types/index.ts` — Container에 compose 필드 추가
- `src/components/containers/ContainerList.tsx` — 그룹핑 로직 + ComposeGroup 렌더링
- `src/components/containers/ContainerRow.tsx` — compose_service 표시 지원 (옵션)
