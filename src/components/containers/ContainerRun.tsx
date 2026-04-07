import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useRunContainer } from "../../hooks/useContainers";

export function ContainerRun() {
  const [image, setImage] = useState("");
  const [name, setName] = useState("");
  const [ports, setPorts] = useState("");
  const [expanded, setExpanded] = useState(false);
  const run = useRunContainer();

  const handleRun = () => {
    if (!image.trim()) return;
    run.mutate({
      image: image.trim(),
      name: name.trim() || undefined,
      ports: ports.trim() || undefined,
    });
    setImage("");
    setName("");
    setPorts("");
    setExpanded(false);
  };

  if (!expanded) {
    return (
      <Button variant="outline" size="sm" onClick={() => setExpanded(true)}>
        Run Container
      </Button>
    );
  }

  return (
    <div className="rounded-md border p-3 space-y-2">
      <div className="flex gap-2">
        <Input
          placeholder="Image (e.g. nginx:alpine)"
          value={image}
          onChange={(e) => setImage(e.target.value)}
          className="flex-1"
        />
        <Input
          placeholder="Name (optional)"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="w-40"
        />
      </div>
      <div className="flex gap-2">
        <Input
          placeholder="Ports (e.g. 8080:80, 3000:3000)"
          value={ports}
          onChange={(e) => setPorts(e.target.value)}
          className="flex-1"
          onKeyDown={(e) => e.key === "Enter" && handleRun()}
        />
        <Button onClick={handleRun} disabled={run.isPending || !image.trim()}>
          {run.isPending ? "Starting..." : "Run"}
        </Button>
        <Button variant="ghost" onClick={() => setExpanded(false)}>
          Cancel
        </Button>
      </div>
    </div>
  );
}
