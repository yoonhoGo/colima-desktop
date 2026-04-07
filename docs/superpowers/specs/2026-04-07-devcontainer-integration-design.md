# Dev Container Integration Design

## Overview

Colima Desktop에 devcontainer 라이프사이클 관리 기능을 추가한다. `@devcontainers/cli`를 활용하여 GUI에서 devcontainer를 빌드/시작/중지/삭제할 수 있도록 한다.

## 요구사항

- **범위**: devcontainer 라이프사이클 관리 (빌드/시작/중지/삭제)
- **CLI**: `@devcontainers/cli` 공식 CLI 활용
- **프로젝트 감지**: 사용자가 수동으로 프로젝트 경로 등록
- **작업 방식**: 연결 정보(docker exec, VS Code 명령어) 표시 및 복사
- **UI 배치**: 기존 Containers 페이지 내 탭으로 통합, 프로젝트별 아코디언

## 아키텍처

### 3계층 구조

```
Frontend (React)          Backend (Rust/Tauri)        External CLI
┌─────────────────┐      ┌─────────────────────┐     ┌──────────────────┐
│ ContainerList    │      │ devcontainer.rs      │     │ devcontainer CLI  │
│  ├─ Containers탭│◄────►│  (Tauri commands)    │────►│  up/build/stop   │
│  └─ DevContainers│      │                     │     │  read-configuration│
│     탭           │      │ config.rs            │     │                  │
│                  │      │  (프로젝트 영속화)    │     │ Docker CLI (기존) │
│ DevContainerTab  │      │                     │────►│  ps/stop/inspect  │
│ DevContainerGroup│      │ CliExecutor (기존)   │     │                  │
│ AddProjectDialog │      │                     │     │                  │
└─────────────────┘      └─────────────────────┘     └──────────────────┘
```

### 핵심 설계 결정

- 프로젝트 등록 정보 → 앱 데이터 디렉토리의 JSON 파일에 영속화
- devcontainer.json 파싱 → `devcontainer read-configuration`에 위임
- 컨테이너 상태 조회 → Docker CLI 재활용 (라벨 기반)
- 빌드 진행률 → Tauri 이벤트 스트리밍 (기존 로그 스트리밍 패턴 재활용)

## UI 구조

### 탭 전환

Containers 페이지 상단에 "Containers | Dev Containers" 탭을 추가한다. 기존 ContainerList 내용은 Containers 탭에, devcontainer 기능은 Dev Containers 탭에 배치한다.

### 프로젝트 아코디언

ComposeGroup 패턴을 재활용한 DevContainerGroup 컴포넌트:

**접힌 상태:**
- 프로젝트명, 워크스페이스 경로, 상태 배지
- 상태별 액션 버튼

**펼친 상태:**
- Config 정보 (이미지, features)
- Connection Info (docker exec 명령어, VS Code 명령어) + 복사 버튼
- 빌드 중일 때: 로그 스트림 표시

### 상태 & 액션 매트릭스

| 상태 | 배지 색상 | 사용 가능 액션 | 연결 정보 |
|------|----------|--------------|----------|
| Running | 초록 | Rebuild, Stop, Remove | 표시 |
| Stopped | 회색 | Start, Remove | 숨김 |
| Not Built | 노랑 | Build, Remove | 숨김 |
| Building | 파랑(스피너) | 없음 (비활성화) | 숨김 |

### Add Project 흐름

1. "Add Project" 버튼 클릭
2. Tauri `dialog.open()` → 폴더 선택
3. 백엔드에서 `.devcontainer/devcontainer.json` 또는 `.devcontainer.json` 존재 여부 검증
4. 성공 시 등록, 실패 시 에러 메시지

## 백엔드 설계

### 데이터 모델

```rust
struct DevContainerProject {
    id: String,                    // UUID
    workspace_path: String,        // 프로젝트 절대 경로
    name: String,                  // 디렉토리명 또는 사용자 지정
    status: DevContainerStatus,    // 실시간 조회
}

enum DevContainerStatus {
    NotBuilt,
    Running,
    Stopped,
    Building,
}
```

### Tauri Commands

| Command | CLI 실행 | 설명 |
|---------|---------|------|
| `list_devcontainer_projects` | 없음 (로컬 JSON 읽기 + Docker 상태 조회) | 등록된 프로젝트 목록 + 실시간 상태 |
| `add_devcontainer_project` | `devcontainer read-configuration` | 프로젝트 등록 + devcontainer.json 검증 |
| `remove_devcontainer_project` | 없음 (옵션: `docker rm`) | 등록 해제 + 컨테이너 삭제 옵션 |
| `devcontainer_build` | `devcontainer build --workspace-folder` | 이미지 빌드 (스트리밍) |
| `devcontainer_up` | `devcontainer up --workspace-folder` | 컨테이너 시작 |
| `devcontainer_stop` | `docker stop <container>` | 컨테이너 중지 |
| `devcontainer_read_config` | `devcontainer read-configuration` | 설정 정보 조회 |

### 프로젝트 영속화

- 저장 위치: Tauri `app_data_dir` / `devcontainer-projects.json`
- 저장 내용: `{ projects: [{ id, workspace_path, name }] }`
- 상태(Running/Stopped)는 저장하지 않음 → 매번 Docker CLI로 실시간 확인

### 빌드 스트리밍

`devcontainer build` stdout을 라인 단위로 읽어 Tauri 이벤트(`devcontainer-build-{id}`)로 emit한다. 기존 `stream_container_logs` 패턴과 동일.

## 프론트엔드 설계

### 신규 파일

```
src/
├── components/containers/
│   ├── DevContainerTab.tsx       # 탭 콘텐츠 (프로젝트 목록 + Add 버튼)
│   ├── DevContainerGroup.tsx     # 프로젝트별 아코디언
│   └── AddProjectDialog.tsx      # 프로젝트 등록 다이얼로그
├── hooks/
│   └── useDevcontainers.ts       # React Query hooks
├── lib/
│   └── tauri.ts                  # (수정) devcontainer API 함수 추가
└── types/
    └── index.ts                  # (수정) DevContainerProject 타입 추가
```

### 수정 파일

- `ContainerList.tsx` — 상단에 탭 UI 추가, 탭 상태 관리

### Hook 구조

```typescript
useDevcontainerProjects()       // 목록 조회 (3초 폴링)
useAddProject()                 // 프로젝트 등록 mutation
useRemoveProject()              // 프로젝트 해제 mutation
useDevcontainerAction()         // build/up/stop 통합 mutation
useDevcontainerConfig(path)     // 설정 정보 조회
```

## 에러 처리

### devcontainer CLI 미설치

Dev Containers 탭 진입 시 `devcontainer --version` 실행으로 확인. 미설치 시 탭 상단에 안내 배너 표시 + `npm install -g @devcontainers/cli` 복사 버튼.

### Colima 미실행 상태

devcontainer CLI는 Docker daemon 필요. Colima가 꺼져 있으면 액션 시도 시 "Colima가 실행 중이지 않습니다. 먼저 시작해주세요." 에러 토스트.

### 빌드 실패

빌드 로그 스트림 그대로 표시, 마지막에 에러 메시지 강조. 상태를 `NotBuilt`으로 유지, "Retry Build" 버튼 표시.

### 프로젝트 경로 유효하지 않음

등록된 경로가 삭제/이동된 경우 목록에서 경고 아이콘 + "경로를 찾을 수 없습니다" 표시. Re-link 또는 Remove 옵션 제공.

### 동시 작업

한 프로젝트에 대해 빌드/시작이 진행 중이면 해당 프로젝트의 액션 버튼 비활성화. `Building...` 스피너 표시.
