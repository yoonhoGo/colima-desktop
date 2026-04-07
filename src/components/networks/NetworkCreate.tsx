import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useCreateNetwork } from "../../hooks/useNetworks";

export function NetworkCreate() {
  const [name, setName] = useState("");
  const [driver, setDriver] = useState("");
  const create = useCreateNetwork();

  const handleCreate = () => {
    if (!name.trim()) return;
    create.mutate({
      name: name.trim(),
      driver: driver.trim() || undefined,
    });
    setName("");
    setDriver("");
  };

  return (
    <div className="flex gap-2">
      <Input
        placeholder="Network name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && handleCreate()}
        className="max-w-xs"
      />
      <Input
        placeholder="Driver (default: bridge)"
        value={driver}
        onChange={(e) => setDriver(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && handleCreate()}
        className="w-44"
      />
      <Button onClick={handleCreate} disabled={create.isPending || !name.trim()}>
        {create.isPending ? "Creating..." : "Create"}
      </Button>
    </div>
  );
}
