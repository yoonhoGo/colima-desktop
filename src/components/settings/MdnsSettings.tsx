import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Loader2, Radio, ArrowRightLeft } from "lucide-react";
import { useMdnsConfig, useMdnsSetConfig, useMdnsStatus } from "../../hooks/useMdns";
import { api } from "../../lib/tauri";
import type { MdnsConfig, ProxyStatus } from "../../types";

export function MdnsSettings() {
  const { data: config, isLoading, error } = useMdnsConfig();
  const { data: status } = useMdnsStatus();
  const saveMutation = useMdnsSetConfig();

  const [enabled, setEnabled] = useState(false);
  const [autoRegister, setAutoRegister] = useState(true);
  const [defaultServiceType, setDefaultServiceType] = useState("_http._tcp.local.");
  const [ipMode, setIpMode] = useState("auto");
  const [customIp, setCustomIp] = useState("");

  useEffect(() => {
    if (config) {
      setEnabled(config.enabled);
      setAutoRegister(config.auto_register);
      setDefaultServiceType(config.default_service_type);
      const mode = config.ip_mode || "auto";
      if (["auto", "vm", "localhost"].includes(mode)) {
        setIpMode(mode);
        setCustomIp("");
      } else {
        setIpMode("custom");
        setCustomIp(mode);
      }
    }
  }, [config]);

  const resolvedIpMode = ipMode === "custom" ? customIp : ipMode;

  const hasChanges = (() => {
    if (!config) return false;
    return (
      enabled !== config.enabled ||
      autoRegister !== config.auto_register ||
      defaultServiceType !== config.default_service_type ||
      resolvedIpMode !== (config.ip_mode || "auto")
    );
  })();

  const handleSave = () => {
    if (!config) return;
    const updated: MdnsConfig = {
      ...config,
      enabled,
      auto_register: autoRegister,
      default_service_type: defaultServiceType,
      ip_mode: resolvedIpMode,
    };
    saveMutation.mutate(updated);
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center text-sm">
        Failed to load mDNS settings
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <div className="flex items-center gap-2">
        <Radio className="h-5 w-5" />
        <h2 className="text-lg font-semibold">mDNS Settings</h2>
      </div>

      <div className="space-y-5">
        <label className="flex items-center gap-2 text-sm font-medium">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(e) => setEnabled(e.target.checked)}
            disabled={saveMutation.isPending}
            className="rounded"
          />
          Enable mDNS Service Publishing
        </label>

        <p className="text-xs text-muted-foreground -mt-3">
          Expose running containers on your local network via mDNS (Bonjour)
        </p>

        <label className="flex items-center gap-2 text-sm font-medium">
          <input
            type="checkbox"
            checked={autoRegister}
            onChange={(e) => setAutoRegister(e.target.checked)}
            disabled={saveMutation.isPending || !enabled}
            className="rounded"
          />
          Auto-register containers with exposed ports
        </label>

        <div className="space-y-2">
          <label className="text-sm font-medium">IP Resolution</label>
          <p className="text-[10px] text-muted-foreground">
            Which IP address to advertise for mDNS services
          </p>
          <div className="flex gap-1 flex-wrap">
            {(["auto", "vm", "localhost", "custom"] as const).map((mode) => (
              <Button
                key={mode}
                variant={ipMode === mode ? "default" : "outline"}
                size="sm"
                disabled={saveMutation.isPending || !enabled}
                onClick={() => setIpMode(mode)}
              >
                {{
                  auto: "LAN IP",
                  vm: "VM IP",
                  localhost: "Localhost",
                  custom: "Custom",
                }[mode]}
              </Button>
            ))}
          </div>
          <p className="text-[10px] text-muted-foreground">
            {{
              auto: "Host LAN IP — accessible from other devices on your network",
              vm: "Colima VM IP — works offline, direct path to containers",
              localhost: "127.0.0.1 — host-only access",
              custom: "Specify a custom IP address",
            }[ipMode]}
          </p>
          {ipMode === "custom" && (
            <Input
              value={customIp}
              onChange={(e) => setCustomIp(e.target.value)}
              disabled={saveMutation.isPending || !enabled}
              placeholder="192.168.1.100"
              className="font-mono text-sm"
            />
          )}
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium">Default Service Type</label>
          <Input
            value={defaultServiceType}
            onChange={(e) => setDefaultServiceType(e.target.value)}
            disabled={saveMutation.isPending || !enabled}
            placeholder="_http._tcp.local."
          />
        </div>

        {status && (
          <div className="glass-card p-3 space-y-1">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Daemon</span>
              <span className={status.daemon_running ? "text-emerald-600" : "text-muted-foreground"}>
                {status.daemon_running ? "Running" : "Stopped"}
              </span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Registered Services</span>
              <span>{status.registered_count}</span>
            </div>
          </div>
        )}
      </div>

      <div className="space-y-2">
        <Button
          onClick={handleSave}
          disabled={!hasChanges || saveMutation.isPending}
          className="w-full"
        >
          {saveMutation.isPending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Saving...
            </>
          ) : (
            "Save"
          )}
        </Button>

        {saveMutation.isError && (
          <p className="text-center text-xs text-destructive">
            Failed to save mDNS settings
          </p>
        )}

        {saveMutation.isSuccess && (
          <p className="text-center text-xs text-green-600">
            Settings saved successfully
          </p>
        )}
      </div>

      {/* Reverse Proxy Section */}
      {enabled && <ReverseProxySection />}
    </div>
  );
}

function ReverseProxySection() {
  const queryClient = useQueryClient();

  const { data: proxyStatus } = useQuery({
    queryKey: ["proxy-status"],
    queryFn: () => api.proxyGetStatus(),
    refetchInterval: 5000,
  });

  const startProxy = useMutation({
    mutationFn: () => api.proxyStart(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["proxy-status"] }),
  });

  const stopProxy = useMutation({
    mutationFn: () => api.proxyStop(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["proxy-status"] }),
  });

  const enablePf = useMutation({
    mutationFn: () => api.proxyEnablePf(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["proxy-status"] }),
  });

  const disablePf = useMutation({
    mutationFn: () => api.proxyDisablePf(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["proxy-status"] }),
  });

  const running = proxyStatus?.running ?? false;
  const pfEnabled = proxyStatus?.pf_enabled ?? false;
  const routes = proxyStatus?.routes ?? [];
  const port = proxyStatus?.port ?? 7080;

  return (
    <>
      <div className="border-t border-[var(--glass-border)] my-4" />

      <div className="flex items-center gap-2 mb-4">
        <ArrowRightLeft className="h-5 w-5" />
        <h2 className="text-lg font-semibold">Reverse Proxy</h2>
      </div>

      <div className="space-y-4">
        <p className="text-xs text-muted-foreground">
          Access containers via <code>http://hostname.local</code> without specifying ports.
          Routes are synced automatically from mDNS services.
        </p>

        <div className="flex items-center gap-3">
          <Button
            size="sm"
            variant={running ? "outline" : "default"}
            onClick={() => (running ? stopProxy.mutate() : startProxy.mutate())}
            disabled={startProxy.isPending || stopProxy.isPending}
          >
            {running ? "Stop Proxy" : `Start Proxy (:${port})`}
          </Button>

          <div className={`h-2 w-2 rounded-full ${running ? "bg-emerald-500" : "bg-muted-foreground/30"}`} />
          <span className="text-xs text-muted-foreground">{running ? "Running" : "Stopped"}</span>
        </div>

        {running && (
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <Button
                size="sm"
                variant={pfEnabled ? "outline" : "default"}
                onClick={() => (pfEnabled ? disablePf.mutate() : enablePf.mutate())}
                disabled={enablePf.isPending || disablePf.isPending}
              >
                {pfEnabled ? "Disable Port 80 → " + port : "Enable Port 80 → " + port}
              </Button>
              <span className="text-[10px] text-muted-foreground">
                {pfEnabled
                  ? "http://hostname.local works (pf active)"
                  : "Requires admin — redirects :80 to proxy"}
              </span>
            </div>

            {routes.length > 0 && (
              <div className="glass-card p-3 space-y-1">
                <div className="text-xs font-medium mb-2">Active Routes</div>
                {routes.map((r) => (
                  <div key={r.hostname} className="flex items-center justify-between text-xs">
                    <code className="text-[11px]">{r.hostname}</code>
                    <span className="text-muted-foreground">→ localhost:{r.target_port}</span>
                  </div>
                ))}
              </div>
            )}

            {routes.length === 0 && (
              <p className="text-[10px] text-muted-foreground">
                No routes yet. Register mDNS services to create routes automatically.
              </p>
            )}
          </div>
        )}
      </div>
    </>
  );
}
