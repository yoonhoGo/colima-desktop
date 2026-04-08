import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Radio } from "lucide-react";
import type { Container, ContainerMdnsConfig } from "../../types";
import { useContainerAction } from "../../hooks/useContainers";
import { useMdnsSetContainerConfig } from "../../hooks/useMdns";

interface ContainerRowProps {
  container: Container;
  onViewLogs: (id: string) => void;
  onInspect?: (id: string) => void;
  showServiceName?: boolean;
  mdnsConfig?: ContainerMdnsConfig;
  mdnsEnabled?: boolean;
}

function parseFirstHostPort(ports: string): number {
  // Format: "0.0.0.0:8080->80/tcp"
  const match = ports.match(/:(\d+)->/);
  return match ? parseInt(match[1], 10) : 0;
}

export function ContainerRow({ container, onViewLogs, onInspect, showServiceName, mdnsConfig, mdnsEnabled }: ContainerRowProps) {
  const action = useContainerAction();
  const setMdnsConfig = useMdnsSetContainerConfig();
  const isRunning = container.state === "running";
  const displayName = showServiceName && container.compose_service
    ? container.compose_service
    : container.name;

  const [showMdnsForm, setShowMdnsForm] = useState(false);
  const [mdnsPort, setMdnsPort] = useState(
    mdnsConfig?.port?.toString() ?? parseFirstHostPort(container.ports).toString()
  );
  const [mdnsServiceType, setMdnsServiceType] = useState(
    mdnsConfig?.service_type ?? "_http._tcp"
  );

  const isMdnsActive = mdnsConfig?.enabled ?? false;

  const handleMdnsToggle = () => {
    if (isMdnsActive) {
      // Disable mDNS for this container
      setMdnsConfig.mutate({
        containerId: container.id,
        containerName: container.name,
        enabled: false,
        serviceType: mdnsConfig?.service_type ?? "_http._tcp",
        port: mdnsConfig?.port ?? 0,
      });
    } else {
      setShowMdnsForm(true);
    }
  };

  const handleMdnsSubmit = () => {
    const port = parseInt(mdnsPort, 10);
    if (isNaN(port) || port <= 0) return;
    setMdnsConfig.mutate({
      containerId: container.id,
      containerName: container.name,
      enabled: true,
      serviceType: mdnsServiceType,
      port,
    }, {
      onSuccess: () => setShowMdnsForm(false),
    });
  };

  return (
    <div className="glass-card px-4 py-3">
      <div className="flex items-center gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-sm truncate">{displayName}</span>
            <Badge variant={isRunning ? "default" : "secondary"} className="text-xs">
              {container.state}
            </Badge>
            {isMdnsActive && (
              <Badge variant="outline" className="text-xs gap-1">
                <Radio className="h-2.5 w-2.5" />
                mDNS
              </Badge>
            )}
          </div>
          <div className="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
            <span className="truncate">{container.image}</span>
            {container.ports && <span>{container.ports}</span>}
            {isMdnsActive && mdnsConfig && (
              <span className="text-blue-500">
                {container.name}.local:{mdnsConfig.port}
              </span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-1">
          {mdnsEnabled && isRunning && (
            <Button
              variant={isMdnsActive ? "default" : "ghost"}
              size="sm"
              onClick={handleMdnsToggle}
              disabled={setMdnsConfig.isPending}
              title={isMdnsActive ? "Disable mDNS" : "Enable mDNS"}
            >
              <Radio className="h-3.5 w-3.5" />
            </Button>
          )}
          {isRunning ? (
            <Button variant="ghost" size="sm" onClick={() => action.mutate({ id: container.id, action: "stop" })} disabled={action.isPending}>Stop</Button>
          ) : (
            <Button variant="ghost" size="sm" onClick={() => action.mutate({ id: container.id, action: "start" })} disabled={action.isPending}>Start</Button>
          )}
          <Button variant="ghost" size="sm" onClick={() => action.mutate({ id: container.id, action: "restart" })} disabled={action.isPending}>Restart</Button>
          <Button variant="ghost" size="sm" onClick={() => onViewLogs(container.id)}>Logs</Button>
          {onInspect && <Button variant="ghost" size="sm" onClick={() => onInspect(container.id)}>Inspect</Button>}
          <Button variant="ghost" size="sm" className="text-destructive" onClick={() => action.mutate({ id: container.id, action: "remove" })} disabled={action.isPending}>Remove</Button>
        </div>
      </div>

      {/* mDNS Configuration Form */}
      {showMdnsForm && (
        <div className="mt-2 flex items-center gap-2 border-t border-[var(--glass-border)] pt-2">
          <span className="text-xs text-muted-foreground whitespace-nowrap">mDNS:</span>
          <Input
            placeholder="Port"
            value={mdnsPort}
            onChange={(e) => setMdnsPort(e.target.value)}
            className="w-20 h-7 text-xs"
            type="number"
          />
          <Input
            placeholder="Service type"
            value={mdnsServiceType}
            onChange={(e) => setMdnsServiceType(e.target.value)}
            className="flex-1 h-7 text-xs"
          />
          <Button
            variant="default"
            size="sm"
            className="h-7 text-xs"
            onClick={handleMdnsSubmit}
            disabled={setMdnsConfig.isPending || !mdnsPort}
          >
            Enable
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-xs"
            onClick={() => setShowMdnsForm(false)}
          >
            Cancel
          </Button>
        </div>
      )}
    </div>
  );
}
