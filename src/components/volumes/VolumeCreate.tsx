import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useCreateVolume } from "../../hooks/useVolumes";

export function VolumeCreate() {
  const [name, setName] = useState("");
  const create = useCreateVolume();

  const handleCreate = () => {
    if (!name.trim()) return;
    create.mutate({ name: name.trim() });
    setName("");
  };

  return (
    <div className="flex gap-2">
      <Input
        placeholder="Volume name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && handleCreate()}
        className="max-w-sm"
      />
      <Button onClick={handleCreate} disabled={create.isPending || !name.trim()}>
        {create.isPending ? "Creating..." : "Create"}
      </Button>
    </div>
  );
}
