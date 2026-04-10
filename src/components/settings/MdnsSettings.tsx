import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Loader2, Radio } from "lucide-react";
import { useMdnsConfig, useMdnsSetConfig, useMdnsStatus } from "../../hooks/useMdns";
import type { MdnsConfig } from "../../types";

export function MdnsSettings() {
  const { data: config, isLoading, error } = useMdnsConfig();
  const { data: status } = useMdnsStatus();
  const saveMutation = useMdnsSetConfig();

  const [enabled, setEnabled] = useState(false);
  const [autoRegister, setAutoRegister] = useState(true);
  const [defaultServiceType, setDefaultServiceType] = useState("_http._tcp.local.");

  useEffect(() => {
    if (config) {
      setEnabled(config.enabled);
      setAutoRegister(config.auto_register);
      setDefaultServiceType(config.default_service_type);
    }
  }, [config]);

  const hasChanges = (() => {
    if (!config) return false;
    return (
      enabled !== config.enabled ||
      autoRegister !== config.auto_register ||
      defaultServiceType !== config.default_service_type
    );
  })();

  const handleSave = () => {
    if (!config) return;
    const updated: MdnsConfig = {
      ...config,
      enabled,
      auto_register: autoRegister,
      default_service_type: defaultServiceType,
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
    </div>
  );
}
