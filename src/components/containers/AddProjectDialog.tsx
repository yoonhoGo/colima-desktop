import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { FolderPlus } from "lucide-react";
import { useAddDevcontainerProject } from "../../hooks/useDevcontainers";

export function AddProjectDialog() {
  const addProject = useAddDevcontainerProject();
  const [error, setError] = useState<string | null>(null);

  const handleAdd = async () => {
    setError(null);
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;

    const path = typeof selected === "string" ? selected : selected[0];
    if (!path) return;

    addProject.mutate(path, {
      onError: (err) => {
        setError(err instanceof Error ? err.message : String(err));
      },
    });
  };

  return (
    <div className="flex items-center gap-2">
      <Button size="sm" onClick={handleAdd} disabled={addProject.isPending}>
        <FolderPlus className="h-4 w-4 mr-1" />
        {addProject.isPending ? "Adding..." : "Add Project"}
      </Button>
      {error && <span className="text-xs text-destructive">{error}</span>}
    </div>
  );
}
