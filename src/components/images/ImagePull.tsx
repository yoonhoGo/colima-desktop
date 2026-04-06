import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { usePullImage } from "../../hooks/useImages";

export function ImagePull() {
  const [name, setName] = useState("");
  const pull = usePullImage();

  const handlePull = () => {
    if (!name.trim()) return;
    pull.mutate(name.trim());
    setName("");
  };

  return (
    <div className="flex gap-2">
      <Input placeholder="Image name (e.g. nginx:alpine)" value={name} onChange={(e) => setName(e.target.value)} onKeyDown={(e) => e.key === "Enter" && handlePull()} className="max-w-sm" />
      <Button onClick={handlePull} disabled={pull.isPending || !name.trim()}>
        {pull.isPending ? "Pulling..." : "Pull"}
      </Button>
    </div>
  );
}
