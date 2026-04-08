import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Loader2, Trash2, Search, Radio } from "lucide-react";
import {
  useMdnsState,
  useMdnsEnable,
  useMdnsDisable,
  useMdnsBrowse,
  useMdnsRegisterContainer,
  useMdnsUnregisterService,
  useMdnsContainerConfigs,
  useMdnsRemoveContainerConfig,
  useMdnsSetAutoRegister,
} from "@/hooks/useMdns";
import type { MdnsServiceEntry } from "@/types";

export function MdnsPanel() {
  const { data: mdnsState, isLoading } = useMdnsState();
  const { data: containerConfigs } = useMdnsContainerConfigs();
  const enableMutation = useMdnsEnable();
  const disableMutation = useMdnsDisable();
  const browseMutation = useMdnsBrowse();
  const registerMutation = useMdnsRegisterContainer();
  const unregisterMutation = useMdnsUnregisterService();
  const removeConfigMutation = useMdnsRemoveContainerConfig();
  const setAutoRegisterMutation = useMdnsSetAutoRegister();

  const [browseType, setBrowseType] = useState("_http._tcp");
  const [discoveredServices, setDiscoveredServices] = useState<MdnsServiceEntry[]>([]);

  const [regName, setRegName] = useState("");
  const [regPort, setRegPort] = useState("");
  const [regType, setRegType] = useState("_http._tcp");

  const enabled = mdnsState?.enabled ?? false;
  const autoRegister = mdnsState?.auto_register ?? false;

  const handleToggle = () => {
    if (enabled) {
      disableMutation.mutate();
    } else {
      enableMutation.mutate();
    }
  };

  const handleAutoRegisterToggle = () => {
    setAutoRegisterMutation.mutate(!autoRegister);
  };

  const handleBrowse = () => {
    browseMutation.mutate(browseType, {
      onSuccess: (data) => setDiscoveredServices(data),
    });
  };

  const handleRegister = () => {
    const port = parseInt(regPort, 10);
    if (!regName.trim() || isNaN(port)) return;
    registerMutation.mutate(
      { containerName: regName.trim(), port, serviceType: regType || undefined },
      {
        onSuccess: () => {
          setRegName("");
          setRegPort("");
        },
      }
    );
  };

  const handleUnregister = (instanceName: string, serviceType: string) => {
    unregisterMutation.mutate({ instanceName, serviceType });
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const enabledConfigs = containerConfigs?.filter((c) => c.enabled) ?? [];

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <h2 className="text-lg font-semibold">mDNS Service Discovery</h2>

      <div className="space-y-5">
        {/* Enable/Disable Toggle */}
        <div className="space-y-2">
          <label className="flex items-center gap-2 text-sm font-medium">
            <input
              type="checkbox"
              checked={enabled}
              onChange={handleToggle}
              disabled={enableMutation.isPending || disableMutation.isPending}
              className="rounded"
            />
            Enable mDNS
          </label>
          <p className="text-xs text-muted-foreground">
            Advertise and discover services on the local network via multicast DNS.
          </p>
        </div>

        {/* Auto-register Toggle */}
        <div className="space-y-2">
          <label className="flex items-center gap-2 text-sm font-medium">
            <input
              type="checkbox"
              checked={autoRegister}
              onChange={handleAutoRegisterToggle}
              disabled={setAutoRegisterMutation.isPending}
              className="rounded"
            />
            Auto-register containers
          </label>
          <p className="text-xs text-muted-foreground">
            Automatically register running containers with exposed ports as mDNS services.
            You can also enable mDNS per container from the Containers page.
          </p>
        </div>

        {/* Per-container mDNS configs */}
        {enabledConfigs.length > 0 && (
          <div className="space-y-2">
            <label className="text-sm font-medium">Container mDNS Configurations</label>
            <div className="space-y-2">
              {enabledConfigs.map((cfg) => (
                <div key={cfg.container_id} className="glass-list-item flex items-center gap-2 p-2">
                  <div className="flex-1 min-w-0">
                    <span className="text-sm font-mono truncate block">
                      {cfg.container_name}.local
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {cfg.service_type} : {cfg.port}
                    </span>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => removeConfigMutation.mutate(cfg.container_id)}
                    disabled={removeConfigMutation.isPending}
                    className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              ))}
            </div>
          </div>
        )}

        {enabled && (
          <>
            {/* Register Container Service */}
            <div className="space-y-2">
              <label className="text-sm font-medium">Quick Register</label>
              <div className="flex items-center gap-2">
                <Input
                  placeholder="Container name"
                  value={regName}
                  onChange={(e) => setRegName(e.target.value)}
                  disabled={registerMutation.isPending}
                  className="flex-1"
                />
                <Input
                  placeholder="Port"
                  value={regPort}
                  onChange={(e) => setRegPort(e.target.value)}
                  disabled={registerMutation.isPending}
                  className="w-20"
                  type="number"
                />
                <Input
                  placeholder="Service type"
                  value={regType}
                  onChange={(e) => setRegType(e.target.value)}
                  disabled={registerMutation.isPending}
                  className="flex-1"
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleRegister}
                  disabled={!regName.trim() || !regPort.trim() || registerMutation.isPending}
                >
                  {registerMutation.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Radio className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </div>

            {/* Registered Services */}
            <div className="space-y-2">
              <label className="text-sm font-medium">Active Services</label>
              {(!mdnsState?.registered_services || mdnsState.registered_services.length === 0) ? (
                <p className="text-xs text-muted-foreground">No services registered.</p>
              ) : (
                <div className="space-y-2">
                  {mdnsState.registered_services.map((svc, index) => {
                    const isAuto = svc.properties.some((p) => p.key === "auto" && p.value === "true");
                    return (
                      <div key={index} className="glass-list-item flex items-center gap-2 p-2">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-1">
                            <span className="text-sm font-mono truncate block">
                              {svc.instance_name}
                            </span>
                            {isAuto && (
                              <span className="text-[10px] text-muted-foreground bg-muted px-1 rounded">auto</span>
                            )}
                          </div>
                          <span className="text-xs text-muted-foreground">
                            {svc.service_type} : {svc.port}
                          </span>
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleUnregister(svc.instance_name, svc.service_type)}
                          disabled={unregisterMutation.isPending}
                          className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>

            {/* Browse Services */}
            <div className="space-y-2">
              <label className="text-sm font-medium">Discover Services</label>
              <div className="flex items-center gap-2">
                <Input
                  placeholder="e.g. _http._tcp"
                  value={browseType}
                  onChange={(e) => setBrowseType(e.target.value)}
                  disabled={browseMutation.isPending}
                  className="flex-1"
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleBrowse();
                  }}
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleBrowse}
                  disabled={!browseType.trim() || browseMutation.isPending}
                >
                  {browseMutation.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Search className="h-4 w-4" />
                  )}
                </Button>
              </div>

              {discoveredServices.length > 0 && (
                <div className="space-y-2">
                  {discoveredServices.map((svc, index) => (
                    <div key={index} className="glass-list-item p-2">
                      <div className="text-sm font-mono truncate">{svc.instance_name}</div>
                      <div className="text-xs text-muted-foreground">
                        {svc.hostname} : {svc.port}
                      </div>
                      {svc.addresses.length > 0 && (
                        <div className="text-xs text-muted-foreground">
                          {svc.addresses.join(", ")}
                        </div>
                      )}
                      {svc.properties.length > 0 && (
                        <div className="text-xs text-muted-foreground mt-1">
                          {svc.properties.map((p) => `${p.key}=${p.value}`).join(", ")}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}

              {browseMutation.isSuccess && discoveredServices.length === 0 && (
                <p className="text-xs text-muted-foreground">No services found.</p>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
