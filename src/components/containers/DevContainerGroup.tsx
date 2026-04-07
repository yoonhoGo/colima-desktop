import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ChevronRight, ChevronDown, Copy, AlertTriangle, Loader2 } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import type { DevContainerProject } from "../../types";
import { useDevcontainerAction, useRemoveDevcontainerProject, useDevcontainerConfig } from "../../hooks/useDevcontainers";

interface DevContainerGroupProps {
  project: DevContainerProject;
}

export function DevContainerGroup({ project }: DevContainerGroupProps) {
  const [expanded, setExpanded] = useState(false);
  const [buildLog, setBuildLog] = useState<string[]>([]);
  const [isBuilding, setIsBuilding] = useState(false);
  const action = useDevcontainerAction();
  const remove = useRemoveDevcontainerProject();
  const { data: config } = useDevcontainerConfig(
    expanded && project.status !== "path_missing" ? project.workspace_path : ""
  );

  const eventKey = project.workspace_path.replace(/\//g, "_");

  useEffect(() => {
    if (!isBuilding) return;

    const unlisten = listen<string>(`devcontainer-log-${eventKey}`, (event) => {
      if (event.payload === "[done]") {
        setIsBuilding(false);
      } else {
        setBuildLog((prev) => [...prev.slice(-200), event.payload]);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [isBuilding, eventKey]);

  const handleAction = (type: "up" | "build" | "stop") => {
    if (type === "up" || type === "build") {
      setIsBuilding(true);
      setBuildLog([]);
      setExpanded(true);
    }
    action.mutate(
      { workspacePath: project.workspace_path, action: type },
      {
        onError: () => setIsBuilding(false),
      }
    );
  };

  const handleRemove = () => {
    remove.mutate({
      id: project.id,
      removeContainer: project.status !== "not_built",
    });
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const statusBadge = () => {
    switch (project.status) {
      case "running":
        return <Badge variant="default" className="text-xs">Running</Badge>;
      case "stopped":
        return <Badge variant="secondary" className="text-xs">Stopped</Badge>;
      case "not_built":
        return <Badge variant="outline" className="text-xs bg-yellow-500/10 text-yellow-500 border-yellow-500/20">Not Built</Badge>;
      case "path_missing":
        return <Badge variant="destructive" className="text-xs">Path Missing</Badge>;
      default:
        return null;
    }
  };

  const actionButtons = () => {
    if (isBuilding) {
      return <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />;
    }

    const disabled = action.isPending || remove.isPending;

    switch (project.status) {
      case "running":
        return (
          <>
            <Button variant="ghost" size="sm" onClick={() => handleAction("build")} disabled={disabled}>Rebuild</Button>
            <Button variant="ghost" size="sm" onClick={() => handleAction("stop")} disabled={disabled}>Stop</Button>
            <Button variant="ghost" size="sm" className="text-destructive" onClick={handleRemove} disabled={disabled}>Remove</Button>
          </>
        );
      case "stopped":
        return (
          <>
            <Button variant="ghost" size="sm" onClick={() => handleAction("up")} disabled={disabled}>Start</Button>
            <Button variant="ghost" size="sm" className="text-destructive" onClick={handleRemove} disabled={disabled}>Remove</Button>
          </>
        );
      case "not_built":
        return (
          <>
            <Button variant="ghost" size="sm" onClick={() => handleAction("up")} disabled={disabled}>Build &amp; Start</Button>
            <Button variant="ghost" size="sm" className="text-destructive" onClick={handleRemove} disabled={disabled}>Remove</Button>
          </>
        );
      case "path_missing":
        return (
          <Button variant="ghost" size="sm" className="text-destructive" onClick={handleRemove} disabled={disabled}>Remove</Button>
        );
      default:
        return null;
    }
  };

  const dockerExecCmd = project.container_id
    ? `docker exec -it ${project.container_id} /bin/bash`
    : "";
  const hexPath = Array.from(new TextEncoder().encode(project.workspace_path)).map(b => b.toString(16).padStart(2, '0')).join('');
  const vscodeCmd = `code --folder-uri vscode-remote://dev-container+${hexPath}/workspaces/${project.name}`;

  return (
    <div className="glass-group overflow-hidden">
      <div
        className="flex items-center gap-2 px-4 py-3 cursor-pointer hover:bg-[var(--glass-bg-hover)] transition-all"
        onClick={() => setExpanded(!expanded)}
      >
        {expanded ? (
          <ChevronDown className="h-4 w-4 text-muted-foreground shrink-0" />
        ) : (
          <ChevronRight className="h-4 w-4 text-muted-foreground shrink-0" />
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="font-medium text-sm truncate">{project.name}</span>
            {statusBadge()}
            {project.status === "path_missing" && (
              <AlertTriangle className="h-3.5 w-3.5 text-destructive" />
            )}
          </div>
          <span className="text-xs text-muted-foreground truncate block">{project.workspace_path}</span>
        </div>
        <div className="ml-auto flex items-center gap-1 shrink-0" onClick={(e) => e.stopPropagation()}>
          {actionButtons()}
        </div>
      </div>

      {expanded && (
        <div className="border-t border-[var(--glass-border)] px-4 pb-3 pt-2 space-y-3">
          {config && (
            <div className="grid grid-cols-2 gap-2">
              <div className="rounded-md bg-muted/30 px-3 py-2">
                <div className="text-[10px] uppercase text-muted-foreground mb-1">Image</div>
                <div className="text-xs truncate">{config.image || "Dockerfile-based"}</div>
              </div>
              <div className="rounded-md bg-muted/30 px-3 py-2">
                <div className="text-[10px] uppercase text-muted-foreground mb-1">Features</div>
                <div className="text-xs truncate">
                  {config.features.length > 0
                    ? config.features.map((f) => f.split("/").pop()).join(", ")
                    : "None"}
                </div>
              </div>
            </div>
          )}

          {project.status === "running" && project.container_id && (
            <div className="rounded-md border border-blue-500/20 bg-blue-500/5 px-3 py-2 space-y-2">
              <div className="text-xs font-semibold text-blue-400">Connection Info</div>
              <div className="flex items-center gap-2">
                <code className="text-[11px] bg-black/30 px-2 py-1 rounded flex-1 truncate">
                  {dockerExecCmd}
                </code>
                <Button variant="ghost" size="sm" className="h-6 px-2" onClick={() => copyToClipboard(dockerExecCmd)}>
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
              <div className="flex items-center gap-2">
                <code className="text-[11px] bg-black/30 px-2 py-1 rounded flex-1 truncate">
                  {vscodeCmd}
                </code>
                <Button variant="ghost" size="sm" className="h-6 px-2" onClick={() => copyToClipboard(vscodeCmd)}>
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
            </div>
          )}

          {(isBuilding || buildLog.length > 0) && (
            <div className="rounded-md bg-black/40 p-2 max-h-48 overflow-y-auto font-mono text-[11px] text-muted-foreground">
              {buildLog.map((line, i) => (
                <div key={i}>{line}</div>
              ))}
              {isBuilding && <Loader2 className="h-3 w-3 animate-spin inline-block mt-1" />}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
