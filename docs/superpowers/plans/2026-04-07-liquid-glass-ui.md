# Liquid Glass UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** colima-desktop의 UI를 macOS Tahoe Liquid Glass (Frosted Light) 디자인으로 전면 개선한다.

**Architecture:** App.css에 glass 디자인 토큰(CSS 변수)과 유틸리티 클래스를 정의한 뒤, 각 컴포넌트의 Tailwind 클래스를 교체하는 방식. CSS 변수 레이어에서 라이트/다크 모드를 모두 처리하므로 컴포넌트 코드 변경을 최소화한다.

**Tech Stack:** Tailwind CSS v4, CSS custom properties, backdrop-filter, CVA (class-variance-authority)

**Design Spec:** `docs/superpowers/specs/2026-04-07-liquid-glass-ui-design.md`

---

## File Structure

| 파일 | 역할 | 변경 유형 |
|------|------|----------|
| `src/App.css` | CSS 변수, glass 유틸 클래스 정의 | 수정 (대폭) |
| `src/components/layout/MainLayout.tsx` | 앱 배경 gradient + glow | 수정 |
| `src/components/layout/Sidebar.tsx` | glass 사이드바 | 수정 |
| `src/components/ui/button.tsx` | CVA variant 업데이트 | 수정 |
| `src/components/ui/badge.tsx` | CVA variant 업데이트 | 수정 |
| `src/components/ui/input.tsx` | glass 배경/border | 수정 |
| `src/components/containers/ContainerRow.tsx` | glass card | 수정 |
| `src/components/containers/ComposeGroup.tsx` | glass group | 수정 |
| `src/components/containers/ContainerRun.tsx` | glass card | 수정 |
| `src/components/containers/ContainerLogs.tsx` | glass panel (터미널은 불투명 유지) | 수정 |
| `src/components/containers/ContainerDetail.tsx` | glass panel sections | 수정 |
| `src/components/containers/ContainerList.tsx` | 변경 없음 (Button 통해 자동 적용) | — |
| `src/components/images/ImageRow.tsx` | glass card | 수정 |
| `src/components/images/ImageList.tsx` | 변경 없음 | — |
| `src/components/images/ImagePull.tsx` | 변경 없음 | — |
| `src/components/volumes/VolumeRow.tsx` | glass card | 수정 |
| `src/components/volumes/VolumeList.tsx` | 변경 없음 | — |
| `src/components/volumes/VolumeCreate.tsx` | 변경 없음 | — |
| `src/components/networks/NetworkRow.tsx` | glass card | 수정 |
| `src/components/networks/NetworkList.tsx` | 변경 없음 | — |
| `src/components/networks/NetworkCreate.tsx` | 변경 없음 | — |
| `src/components/settings/VmSettings.tsx` | glass panel + 슬라이더 | 수정 |
| `src/components/settings/MountSettings.tsx` | glass panel + list items | 수정 |
| `src/components/settings/NetworkSettingsPanel.tsx` | glass panel + list items | 수정 |
| `src/components/settings/DockerSettingsPanel.tsx` | glass panel + list items | 수정 |
| `src/components/settings/UpdatePanel.tsx` | glass panel sections | 수정 |

---

### Task 1: App.css — CSS 변수 및 Glass 유틸리티 클래스

**Files:**
- Modify: `src/App.css`

- [ ] **Step 1: :root 및 .dark CSS 변수 교체**

`src/App.css`의 `:root` 블록과 `.dark` 블록을 다음으로 교체한다:

```css
:root {
    --background: oklch(0.95 0.01 260);
    --foreground: oklch(0.145 0.02 260);
    --card: rgba(255, 255, 255, 0.42);
    --card-foreground: oklch(0.145 0.02 260);
    --popover: rgba(255, 255, 255, 0.65);
    --popover-foreground: oklch(0.145 0.02 260);
    --primary: oklch(0.45 0.15 260);
    --primary-foreground: oklch(0.985 0 0);
    --secondary: rgba(255, 255, 255, 0.35);
    --secondary-foreground: oklch(0.3 0.02 260);
    --muted: rgba(255, 255, 255, 0.25);
    --muted-foreground: oklch(0.45 0.01 260);
    --accent: rgba(255, 255, 255, 0.55);
    --accent-foreground: oklch(0.2 0.02 260);
    --destructive: oklch(0.577 0.245 27.325);
    --border: rgba(255, 255, 255, 0.55);
    --input: rgba(255, 255, 255, 0.45);
    --ring: oklch(0.55 0.15 260);
    --chart-1: oklch(0.6 0.15 260);
    --chart-2: oklch(0.55 0.12 300);
    --chart-3: oklch(0.65 0.1 150);
    --chart-4: oklch(0.6 0.15 30);
    --chart-5: oklch(0.5 0.1 200);
    --radius: 0.875rem;
    --sidebar: rgba(255, 255, 255, 0.42);
    --sidebar-foreground: oklch(0.145 0.02 260);
    --sidebar-primary: oklch(0.45 0.15 260);
    --sidebar-primary-foreground: oklch(0.985 0 0);
    --sidebar-accent: rgba(255, 255, 255, 0.55);
    --sidebar-accent-foreground: oklch(0.2 0.02 260);
    --sidebar-border: rgba(255, 255, 255, 0.45);
    --sidebar-ring: oklch(0.55 0.15 260);

    /* Glass tokens */
    --glass-bg: rgba(255, 255, 255, 0.42);
    --glass-bg-hover: rgba(255, 255, 255, 0.52);
    --glass-bg-active: rgba(255, 255, 255, 0.6);
    --glass-border: rgba(255, 255, 255, 0.55);
    --glass-border-strong: rgba(255, 255, 255, 0.65);
    --glass-blur: blur(20px) saturate(180%);
    --glass-blur-heavy: blur(24px) saturate(180%);
    --glass-shadow: 0 2px 12px rgba(0, 0, 0, 0.04);
    --glass-shadow-hover: 0 4px 20px rgba(0, 0, 0, 0.06);
    --glass-inset: inset 0 1px 0 rgba(255, 255, 255, 0.65);
    --glass-inset-sidebar: inset -1px 0 0 rgba(255, 255, 255, 0.25);

    /* Background gradient */
    --bg-gradient: linear-gradient(145deg, #e8eaf0 0%, #d4d8e0 30%, #c8cdd6 60%, #e0e4ec 100%);
    --bg-glow: radial-gradient(circle at 25% 35%, rgba(147,197,253,0.2) 0%, transparent 50%),
               radial-gradient(circle at 75% 65%, rgba(196,181,253,0.12) 0%, transparent 50%);

    /* Status colors */
    --status-running-bg: rgba(34, 197, 94, 0.1);
    --status-running-text: #16a34a;
    --status-running-border: rgba(34, 197, 94, 0.2);
    --status-running-glow: 0 0 6px rgba(34, 197, 94, 0.4);
    --status-stopped-bg: rgba(0, 0, 0, 0.04);
    --status-stopped-text: #999;
    --status-stopped-border: rgba(0, 0, 0, 0.08);
}

.dark {
    --background: oklch(0.12 0.02 260);
    --foreground: oklch(0.92 0 0);
    --card: rgba(255, 255, 255, 0.04);
    --card-foreground: oklch(0.92 0 0);
    --popover: rgba(255, 255, 255, 0.08);
    --popover-foreground: oklch(0.92 0 0);
    --primary: oklch(0.7 0.15 260);
    --primary-foreground: oklch(0.12 0.02 260);
    --secondary: rgba(255, 255, 255, 0.06);
    --secondary-foreground: oklch(0.92 0 0);
    --muted: rgba(255, 255, 255, 0.06);
    --muted-foreground: oklch(0.6 0 0);
    --accent: rgba(255, 255, 255, 0.08);
    --accent-foreground: oklch(0.92 0 0);
    --destructive: oklch(0.704 0.191 22.216);
    --border: rgba(255, 255, 255, 0.08);
    --input: rgba(255, 255, 255, 0.1);
    --ring: oklch(0.5 0.12 260);
    --chart-1: oklch(0.7 0.15 260);
    --chart-2: oklch(0.65 0.12 300);
    --chart-3: oklch(0.7 0.1 150);
    --chart-4: oklch(0.65 0.15 30);
    --chart-5: oklch(0.6 0.1 200);
    --sidebar: rgba(255, 255, 255, 0.04);
    --sidebar-foreground: oklch(0.92 0 0);
    --sidebar-primary: oklch(0.65 0.2 264);
    --sidebar-primary-foreground: oklch(0.92 0 0);
    --sidebar-accent: rgba(255, 255, 255, 0.08);
    --sidebar-accent-foreground: oklch(0.92 0 0);
    --sidebar-border: rgba(255, 255, 255, 0.08);
    --sidebar-ring: oklch(0.5 0.12 260);

    /* Glass tokens */
    --glass-bg: rgba(255, 255, 255, 0.04);
    --glass-bg-hover: rgba(255, 255, 255, 0.08);
    --glass-bg-active: rgba(255, 255, 255, 0.12);
    --glass-border: rgba(255, 255, 255, 0.08);
    --glass-border-strong: rgba(255, 255, 255, 0.15);
    --glass-blur: blur(24px) saturate(180%);
    --glass-blur-heavy: blur(30px) saturate(180%);
    --glass-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
    --glass-shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.3);
    --glass-inset: inset 0 1px 0 rgba(255, 255, 255, 0.04);
    --glass-inset-sidebar: inset -1px 0 0 rgba(255, 255, 255, 0.04);

    /* Background gradient */
    --bg-gradient: linear-gradient(145deg, #0f0f23 0%, #1a1a3e 50%, #0d1b2a 100%);
    --bg-glow: radial-gradient(circle at 30% 40%, rgba(99,102,241,0.08) 0%, transparent 50%),
               radial-gradient(circle at 70% 60%, rgba(139,92,246,0.06) 0%, transparent 50%);

    /* Status colors */
    --status-running-bg: rgba(74, 222, 128, 0.12);
    --status-running-text: #4ade80;
    --status-running-border: rgba(74, 222, 128, 0.2);
    --status-running-glow: 0 0 6px rgba(74, 222, 128, 0.3);
    --status-stopped-bg: rgba(255, 255, 255, 0.06);
    --status-stopped-text: #888;
    --status-stopped-border: rgba(255, 255, 255, 0.08);
}
```

- [ ] **Step 2: @theme inline 블록에 새 CSS 변수 매핑 추가**

`@theme inline` 블록에 glass 관련 변수 매핑을 추가한다:

```css
@theme inline {
    /* 기존 매핑 유지 + 아래 추가 */
    --color-glass-bg: var(--glass-bg);
    --color-glass-bg-hover: var(--glass-bg-hover);
    --color-glass-bg-active: var(--glass-bg-active);
    --color-glass-border: var(--glass-border);
    --color-glass-border-strong: var(--glass-border-strong);
    --color-status-running-bg: var(--status-running-bg);
    --color-status-running-text: var(--status-running-text);
    --color-status-running-border: var(--status-running-border);
    --color-status-stopped-bg: var(--status-stopped-bg);
    --color-status-stopped-text: var(--status-stopped-text);
    --color-status-stopped-border: var(--status-stopped-border);
}
```

- [ ] **Step 3: @layer base 블록 업데이트**

body에 gradient 배경과 접근성 대응을 추가한다:

```css
@layer base {
  * {
    @apply border-border outline-ring/50;
  }
  body {
    @apply text-foreground;
    background: var(--bg-gradient);
  }
  body::before {
    content: '';
    position: fixed;
    inset: 0;
    background: var(--bg-glow);
    pointer-events: none;
    z-index: 0;
  }
  html {
    @apply font-sans;
  }

  /* Accessibility: reduce transparency */
  @media (prefers-reduced-transparency: reduce) {
    :root {
      --glass-bg: rgba(240, 240, 245, 0.95);
      --glass-bg-hover: rgba(240, 240, 245, 0.98);
      --glass-bg-active: rgba(240, 240, 245, 1);
      --glass-border: rgba(200, 200, 210, 0.8);
    }
    .dark {
      --glass-bg: rgba(25, 25, 45, 0.95);
      --glass-bg-hover: rgba(25, 25, 45, 0.98);
      --glass-bg-active: rgba(25, 25, 45, 1);
      --glass-border: rgba(60, 60, 80, 0.8);
    }
  }
}
```

- [ ] **Step 4: Glass 유틸리티 클래스 추가**

App.css 맨 끝에 추가한다:

```css
@layer utilities {
  .glass-panel {
    background: var(--glass-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow), var(--glass-inset);
    transition: background 0.15s ease, box-shadow 0.15s ease;
  }
  .glass-panel:hover {
    background: var(--glass-bg-hover);
    box-shadow: var(--glass-shadow-hover), var(--glass-inset);
  }

  .glass-card {
    background: var(--glass-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius);
    box-shadow: var(--glass-shadow), var(--glass-inset);
    transition: background 0.15s ease, box-shadow 0.15s ease;
  }
  .glass-card:hover {
    background: var(--glass-bg-hover);
    box-shadow: var(--glass-shadow-hover), var(--glass-inset);
  }

  .glass-sidebar {
    background: var(--glass-bg);
    backdrop-filter: var(--glass-blur-heavy);
    -webkit-backdrop-filter: var(--glass-blur-heavy);
    border-right: 1px solid var(--glass-border);
    box-shadow: var(--glass-inset-sidebar);
  }

  .glass-nav-item {
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    transition: all 0.15s ease;
  }
  .glass-nav-item[data-active="true"] {
    background: var(--glass-bg-active);
    border-color: var(--glass-border-strong);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05), var(--glass-inset);
  }

  .glass-section {
    background: var(--glass-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius);
    box-shadow: var(--glass-shadow), var(--glass-inset);
  }

  .glass-input {
    background: var(--glass-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
  }

  .glass-group {
    background: rgba(255, 255, 255, 0.35);
    backdrop-filter: blur(16px) saturate(160%);
    -webkit-backdrop-filter: blur(16px) saturate(160%);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.03), var(--glass-inset);
  }
  .dark .glass-group {
    background: rgba(255, 255, 255, 0.03);
  }

  .glass-group-child {
    background: rgba(255, 255, 255, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.4);
    box-shadow: 0 1px 6px rgba(0, 0, 0, 0.02), inset 0 1px 0 rgba(255, 255, 255, 0.5);
  }
  .dark .glass-group-child {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    box-shadow: 0 1px 6px rgba(0, 0, 0, 0.15), inset 0 1px 0 rgba(255, 255, 255, 0.02);
  }

  .glass-list-item {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    transition: background 0.12s ease;
  }
  .glass-list-item:hover {
    background: var(--glass-bg-hover);
  }
}
```

- [ ] **Step 5: 확인 — dev server 실행**

Run: `cd /Users/yoonho.go/workspace/colima-desktop && npm run dev`

Expected: 브라우저에서 gradient 배경이 보이고, glow 오버레이가 표시됨. 컴포넌트는 아직 이전 스타일.

- [ ] **Step 6: Commit**

```bash
git add src/App.css
git commit -m "style: Liquid Glass CSS 변수 및 유틸리티 클래스 정의"
```

---

### Task 2: MainLayout — 앱 배경 구조

**Files:**
- Modify: `src/components/layout/MainLayout.tsx`

- [ ] **Step 1: MainLayout 최외곽 div에 relative + z-index 추가**

현재:
```tsx
<div className="flex h-screen">
```

변경:
```tsx
<div className="relative z-10 flex h-screen">
```

이것만 변경하면 body의 gradient 배경과 ::before glow가 자연스럽게 보인다.

- [ ] **Step 2: Settings 탭 바를 glass 스타일로 변경**

현재 탭 그룹 컨테이너:
```tsx
<div className="mx-auto max-w-lg flex gap-1 rounded-lg bg-muted p-1">
```

변경:
```tsx
<div className="mx-auto max-w-lg flex gap-1 rounded-xl glass-panel p-1">
```

현재 탭 버튼 (active 상태):
```tsx
className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
  settingsTab === "vm"
    ? "bg-background text-foreground shadow-sm"
    : "text-muted-foreground hover:text-foreground"
}`}
```

변경:
```tsx
className={`flex-1 rounded-lg px-3 py-1.5 text-sm font-medium transition-all ${
  settingsTab === "vm"
    ? "bg-[var(--glass-bg-active)] text-foreground shadow-sm border border-[var(--glass-border-strong)]"
    : "text-muted-foreground hover:text-foreground border border-transparent"
}`}
```

이 패턴을 5개 탭 버튼 모두에 동일하게 적용한다.

- [ ] **Step 3: Commit**

```bash
git add src/components/layout/MainLayout.tsx
git commit -m "style: MainLayout에 glass 배경 및 탭 바 스타일 적용"
```

---

### Task 3: Sidebar — Glass 사이드바

**Files:**
- Modify: `src/components/layout/Sidebar.tsx`

- [ ] **Step 1: 사이드바 컨테이너 glass 적용**

현재:
```tsx
<div className="flex h-full w-52 flex-col border-r bg-muted/30 p-3">
```

변경:
```tsx
<div className="glass-sidebar flex h-full w-52 flex-col p-3">
```

- [ ] **Step 2: 로고 영역 status dot에 glow 추가**

현재:
```tsx
<div className={cn("h-2 w-2 rounded-full", status?.running ? "bg-green-500" : "bg-gray-400")} />
```

변경:
```tsx
<div className={cn(
  "h-2 w-2 rounded-full",
  status?.running
    ? "bg-[var(--status-running-text)]"
    : "bg-gray-400"
)} style={status?.running ? { boxShadow: 'var(--status-running-glow)' } : undefined} />
```

- [ ] **Step 3: Running 배지를 status 색상으로 변경**

현재:
```tsx
<Badge variant={status?.running ? "default" : "secondary"} className="ml-auto text-xs">
```

변경:
```tsx
<Badge variant={status?.running ? "default" : "secondary"} className={cn(
  "ml-auto text-xs",
  status?.running && "bg-[var(--status-running-bg)] text-[var(--status-running-text)] border border-[var(--status-running-border)]"
)}>
```

- [ ] **Step 4: 네비게이션 아이템 glass 스타일 적용**

현재:
```tsx
<button
  onClick={() => onPageChange("containers")}
  className={cn("rounded-md px-3 py-2 text-left text-sm transition-colors",
    activePage === "containers" ? "bg-accent text-accent-foreground" : "hover:bg-accent/50"
  )}
>
```

변경:
```tsx
<button
  onClick={() => onPageChange("containers")}
  data-active={activePage === "containers"}
  className={cn("glass-nav-item rounded-lg px-3 py-2 text-left text-sm",
    activePage === "containers" ? "text-accent-foreground font-medium" : "text-muted-foreground hover:text-foreground"
  )}
>
```

5개 네비게이션 버튼 모두 동일하게 변경한다.

- [ ] **Step 5: Commit**

```bash
git add src/components/layout/Sidebar.tsx
git commit -m "style: Sidebar에 glass 효과 적용"
```

---

### Task 4: Button — CVA variant Glass 업데이트

**Files:**
- Modify: `src/components/ui/button.tsx`

- [ ] **Step 1: buttonVariants CVA 값 변경**

현재 `buttonVariants`의 base 클래스에서 `rounded-lg` 유지, variant 값만 변경:

```typescript
const buttonVariants = cva(
  "group/button inline-flex shrink-0 items-center justify-center rounded-lg border border-transparent bg-clip-padding text-sm font-medium whitespace-nowrap transition-all outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 active:not-aria-[haspopup]:translate-y-px disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      variant: {
        default: "bg-[var(--glass-bg-active)] text-foreground border-[var(--glass-border-strong)] shadow-sm [a]:hover:bg-[var(--glass-bg-hover)]",
        outline:
          "border-[var(--glass-border)] bg-[var(--glass-bg)] hover:bg-[var(--glass-bg-hover)] hover:text-foreground aria-expanded:bg-[var(--glass-bg-hover)] aria-expanded:text-foreground",
        secondary:
          "bg-secondary text-secondary-foreground hover:bg-[var(--glass-bg-hover)] aria-expanded:bg-secondary aria-expanded:text-secondary-foreground",
        ghost:
          "hover:bg-[var(--glass-bg)] hover:text-foreground aria-expanded:bg-[var(--glass-bg)] aria-expanded:text-foreground",
        destructive:
          "bg-destructive/10 text-destructive hover:bg-destructive/20 focus-visible:border-destructive/40 focus-visible:ring-destructive/20 dark:bg-destructive/20 dark:hover:bg-destructive/30 dark:focus-visible:ring-destructive/40",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default:
          "h-8 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
        xs: "h-6 gap-1 rounded-[min(var(--radius-md),10px)] px-2 text-xs in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3",
        sm: "h-7 gap-1 rounded-[min(var(--radius-md),12px)] px-2.5 text-[0.8rem] in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3.5",
        lg: "h-9 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
        icon: "size-8",
        "icon-xs":
          "size-6 rounded-[min(var(--radius-md),10px)] in-data-[slot=button-group]:rounded-lg [&_svg:not([class*='size-'])]:size-3",
        "icon-sm":
          "size-7 rounded-[min(var(--radius-md),12px)] in-data-[slot=button-group]:rounded-lg",
        "icon-lg": "size-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)
```

size variants는 변경하지 않는다.

- [ ] **Step 2: Commit**

```bash
git add src/components/ui/button.tsx
git commit -m "style: Button CVA variants를 glass 스타일로 업데이트"
```

---

### Task 5: Badge — CVA variant Glass 업데이트

**Files:**
- Modify: `src/components/ui/badge.tsx`

- [ ] **Step 1: badgeVariants CVA 값 변경**

variant 값만 변경한다:

```typescript
const badgeVariants = cva(
  "group/badge inline-flex h-5 w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-4xl border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:size-3!",
  {
    variants: {
      variant: {
        default: "bg-[var(--status-running-bg)] text-[var(--status-running-text)] border-[var(--status-running-border)]",
        secondary:
          "bg-[var(--status-stopped-bg)] text-[var(--status-stopped-text)] border-[var(--status-stopped-border)]",
        destructive:
          "bg-destructive/10 text-destructive focus-visible:ring-destructive/20 dark:bg-destructive/20 dark:focus-visible:ring-destructive/40 [a]:hover:bg-destructive/20",
        outline:
          "border-[var(--glass-border)] text-foreground [a]:hover:bg-[var(--glass-bg)] [a]:hover:text-muted-foreground",
        ghost:
          "hover:bg-[var(--glass-bg)] hover:text-muted-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)
```

- [ ] **Step 2: Commit**

```bash
git add src/components/ui/badge.tsx
git commit -m "style: Badge CVA variants를 glass 스타일로 업데이트"
```

---

### Task 6: Input — Glass 스타일 적용

**Files:**
- Modify: `src/components/ui/input.tsx`

- [ ] **Step 1: Input className 변경**

현재:
```tsx
className={cn(
  "h-8 w-full min-w-0 rounded-lg border border-input bg-transparent px-2.5 py-1 text-base transition-colors outline-none file:inline-flex file:h-6 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-input/50 disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 md:text-sm dark:bg-input/30 dark:disabled:bg-input/80 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40",
  className
)}
```

변경:
```tsx
className={cn(
  "glass-input h-8 w-full min-w-0 rounded-lg px-2.5 py-1 text-base transition-all outline-none file:inline-flex file:h-6 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:border-[var(--glass-border-strong)] focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 md:text-sm dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40",
  className
)}
```

핵심 변경: `border border-input bg-transparent` → `glass-input` (유틸 클래스가 배경/border/blur 처리), `dark:bg-input/30` 제거 (CSS 변수가 처리), focus border를 glass-border-strong으로.

- [ ] **Step 2: Commit**

```bash
git add src/components/ui/input.tsx
git commit -m "style: Input에 glass 스타일 적용"
```

---

### Task 7: ContainerRow, ImageRow, VolumeRow, NetworkRow — Glass Card 적용

**Files:**
- Modify: `src/components/containers/ContainerRow.tsx`
- Modify: `src/components/images/ImageRow.tsx`
- Modify: `src/components/volumes/VolumeRow.tsx`
- Modify: `src/components/networks/NetworkRow.tsx`

이 4개 파일은 모두 동일한 패턴 `rounded-md border px-4 py-3`을 사용한다.

- [ ] **Step 1: ContainerRow 변경**

현재 (줄 21):
```tsx
<div className="flex items-center gap-3 rounded-md border px-4 py-3">
```

변경:
```tsx
<div className="glass-card flex items-center gap-3 px-4 py-3">
```

- [ ] **Step 2: ImageRow 변경**

현재 (줄 14):
```tsx
<div className="flex items-center gap-3 rounded-md border px-4 py-3">
```

변경:
```tsx
<div className="glass-card flex items-center gap-3 px-4 py-3">
```

- [ ] **Step 3: VolumeRow 변경**

현재 (줄 14):
```tsx
<div className="flex items-center gap-3 rounded-md border px-4 py-3">
```

변경:
```tsx
<div className="glass-card flex items-center gap-3 px-4 py-3">
```

- [ ] **Step 4: NetworkRow 변경**

현재 (줄 17):
```tsx
<div className="flex items-center gap-3 rounded-md border px-4 py-3">
```

변경:
```tsx
<div className="glass-card flex items-center gap-3 px-4 py-3">
```

- [ ] **Step 5: Commit**

```bash
git add src/components/containers/ContainerRow.tsx src/components/images/ImageRow.tsx src/components/volumes/VolumeRow.tsx src/components/networks/NetworkRow.tsx
git commit -m "style: Row 컴포넌트들에 glass-card 스타일 적용"
```

---

### Task 8: ComposeGroup — Glass Group 스타일

**Files:**
- Modify: `src/components/containers/ComposeGroup.tsx`

- [ ] **Step 1: 외부 컨테이너 변경**

현재 (줄 37):
```tsx
<div className="rounded-md border">
```

변경:
```tsx
<div className="glass-group overflow-hidden">
```

- [ ] **Step 2: 헤더 hover 배경 변경**

현재 (줄 39):
```tsx
className="flex items-center gap-2 px-4 py-3 cursor-pointer hover:bg-muted/50 transition-colors"
```

변경:
```tsx
className="flex items-center gap-2 px-4 py-3 cursor-pointer hover:bg-[var(--glass-bg-hover)] transition-all"
```

- [ ] **Step 3: Compose 배지에 accent 색상 적용**

현재 (줄 48):
```tsx
<Badge variant={allRunning ? "default" : allStopped ? "secondary" : "outline"} className="text-xs">
```

이것은 그대로 유지한다. Badge의 CVA가 이미 glass 스타일을 적용함.

- [ ] **Step 4: 자식 컨테이너 border-top 변경**

현재 (줄 67):
```tsx
<div className="border-t">
```

변경:
```tsx
<div className="border-t border-[var(--glass-border)]">
```

- [ ] **Step 5: 자식 ContainerRow에 glass-group-child 적용**

자식 ContainerRow는 `<div className="pl-6">` 안에 렌더링된다. ContainerRow 내부에서 이미 glass-card가 적용되지만, ComposeGroup 자식은 한 단계 낮은 glass 레벨이 필요하다.

pl-6 래퍼에 클래스를 추가한다:

현재 (줄 69-71):
```tsx
<div key={container.id} className="pl-6">
  <ContainerRow container={container} onViewLogs={onViewLogs} onInspect={onInspect} showServiceName />
</div>
```

변경:
```tsx
<div key={container.id} className="pl-6 [&>.glass-card]:glass-group-child">
  <ContainerRow container={container} onViewLogs={onViewLogs} onInspect={onInspect} showServiceName />
</div>
```

만약 Tailwind의 arbitrary variant가 유틸 클래스 체이닝을 지원하지 않으면, 대안으로 ContainerRow에 optional className prop을 추가하거나, ComposeGroup 자식 영역 전체에 CSS 스코프를 적용한다. 가장 간단한 방법은 children wrapper에 스타일을 적용하는 것:

```tsx
<div className="border-t border-[var(--glass-border)] px-2 pb-2 pt-1 space-y-1">
  {containers.map((container) => (
    <div key={container.id} className="pl-4">
      <ContainerRow container={container} onViewLogs={onViewLogs} onInspect={onInspect} showServiceName />
    </div>
  ))}
</div>
```

- [ ] **Step 6: Commit**

```bash
git add src/components/containers/ComposeGroup.tsx
git commit -m "style: ComposeGroup에 glass-group 스타일 적용"
```

---

### Task 9: ContainerRun, ContainerLogs, ContainerDetail — Glass 패널

**Files:**
- Modify: `src/components/containers/ContainerRun.tsx`
- Modify: `src/components/containers/ContainerLogs.tsx`
- Modify: `src/components/containers/ContainerDetail.tsx`

- [ ] **Step 1: ContainerRun 변경**

현재 (줄 35):
```tsx
<div className="rounded-md border p-3 space-y-2">
```

변경:
```tsx
<div className="glass-section p-3 space-y-2">
```

- [ ] **Step 2: ContainerLogs — 로그 터미널 영역**

현재 (줄 40):
```tsx
<ScrollArea className="flex-1 rounded-md border bg-black p-3">
```

변경 (터미널은 불투명 유지하되 radius 증가):
```tsx
<ScrollArea className="flex-1 rounded-xl border border-[var(--glass-border)] bg-black/90 p-3 shadow-lg">
```

- [ ] **Step 3: ContainerDetail — 섹션 박스들 변경**

현재 (줄 51, 그리고 여러 반복):
```tsx
<section className="rounded-md border p-4">
```

모든 `rounded-md border p-4` 패턴을 변경:
```tsx
<section className="glass-section p-4">
```

ContainerDetail.tsx에서 이 패턴이 나타나는 모든 `<section>` 태그에 적용한다.

- [ ] **Step 4: Commit**

```bash
git add src/components/containers/ContainerRun.tsx src/components/containers/ContainerLogs.tsx src/components/containers/ContainerDetail.tsx
git commit -m "style: Container 패널 컴포넌트들에 glass 스타일 적용"
```

---

### Task 10: Settings 패널들 — Glass 스타일 적용

**Files:**
- Modify: `src/components/settings/VmSettings.tsx`
- Modify: `src/components/settings/MountSettings.tsx`
- Modify: `src/components/settings/NetworkSettingsPanel.tsx`
- Modify: `src/components/settings/DockerSettingsPanel.tsx`
- Modify: `src/components/settings/UpdatePanel.tsx`

모든 Settings 패널은 동일한 패턴을 공유한다:
- 리스트 아이템: `rounded-md border p-2`
- 정보 박스: `rounded-md border p-4`
- 읽기 전용 필드: `rounded-md border px-3 py-1.5`

- [ ] **Step 1: VmSettings — 슬라이더 스타일은 유지, 전체 감싸는 패널 변경 없음 (이미 space-y 구조)**

VmSettings에는 `rounded-md border` 패턴이 없으므로 변경 불필요. Button과 Input이 이미 glass 스타일.

- [ ] **Step 2: MountSettings — 마운트 항목 리스트 아이템 변경**

현재 (줄 96):
```tsx
<div className="flex items-center gap-2 rounded-md border p-2">
```

변경:
```tsx
<div className="glass-list-item flex items-center gap-2 p-2">
```

- [ ] **Step 3: NetworkSettingsPanel — DNS/호스트 항목 리스트 아이템 변경**

현재 (줄 119):
```tsx
<div className="flex items-center gap-2 rounded-md border p-2">
```

변경:
```tsx
<div className="glass-list-item flex items-center gap-2 p-2">
```

줄 164도 동일 패턴:
```tsx
<div className="flex items-center gap-2 rounded-md border p-2">
```

변경:
```tsx
<div className="glass-list-item flex items-center gap-2 p-2">
```

- [ ] **Step 4: DockerSettingsPanel — 레지스트리/미러 읽기 전용 필드**

현재 (줄 86, 126):
```tsx
<span className="flex-1 rounded-md border px-3 py-1.5 text-sm">
```

변경:
```tsx
<span className="glass-list-item flex-1 px-3 py-1.5 text-sm">
```

- [ ] **Step 5: UpdatePanel — 정보 박스들 변경**

현재 (줄 32):
```tsx
<div className="mt-2 rounded-md border p-4 space-y-3">
```

변경:
```tsx
<div className="mt-2 glass-section p-4 space-y-3">
```

현재 (줄 63):
```tsx
<div className="mt-2 rounded-md border p-4">
```

변경:
```tsx
<div className="mt-2 glass-section p-4">
```

현재 (줄 81):
```tsx
<div className="mt-2 rounded-md border p-4 space-y-3">
```

변경:
```tsx
<div className="mt-2 glass-section p-4 space-y-3">
```

에러 박스 (줄 21):
```tsx
<div className="mx-auto max-w-lg rounded-md border border-destructive p-4 text-destructive">
```

변경:
```tsx
<div className="mx-auto max-w-lg glass-section border-destructive p-4 text-destructive">
```

commit hash 배지 (줄 39):
```tsx
<span className="text-xs font-mono bg-muted px-2 py-0.5 rounded">
```

변경:
```tsx
<span className="text-xs font-mono bg-[var(--glass-bg)] px-2 py-0.5 rounded-md border border-[var(--glass-border)]">
```

- [ ] **Step 6: Commit**

```bash
git add src/components/settings/VmSettings.tsx src/components/settings/MountSettings.tsx src/components/settings/NetworkSettingsPanel.tsx src/components/settings/DockerSettingsPanel.tsx src/components/settings/UpdatePanel.tsx
git commit -m "style: Settings 패널들에 glass 스타일 적용"
```

---

### Task 11: 최종 시각 검증 및 미세 조정

**Files:**
- 잠재적으로 위 모든 파일

- [ ] **Step 1: Dev server에서 전체 페이지 시각 점검**

Run: `cd /Users/yoonho.go/workspace/colima-desktop && npm run dev`

모든 5개 페이지(Containers, Images, Volumes, Networks, Settings) + 5개 Settings 탭을 순회하며 다음을 확인한다:

1. gradient 배경이 제대로 보이는지
2. glass 효과(blur, 반투명)가 사이드바와 카드에 적용되는지
3. hover 효과가 자연스러운지
4. 텍스트 가독성이 충분한지 (대비비)
5. 다크 모드 (`prefers-color-scheme: dark` 또는 `.dark` 클래스)가 정상인지

- [ ] **Step 2: 발견된 문제 수정**

시각 점검에서 발견된 문제를 수정한다. 일반적인 문제:
- backdrop-filter가 적용되지 않으면 `-webkit-backdrop-filter` prefix 확인
- 텍스트 대비가 부족하면 glass-bg 불투명도 증가
- shadow가 과하면 값 조정

- [ ] **Step 3: Tauri 빌드 확인**

Run: `cd /Users/yoonho.go/workspace/colima-desktop && npm run tauri dev`

Tauri WebView에서도 동일하게 보이는지 확인한다. WebKit 기반이므로 `-webkit-backdrop-filter`가 필수.

- [ ] **Step 4: 최종 Commit**

```bash
git add -A
git commit -m "style: Liquid Glass UI 미세 조정"
```
