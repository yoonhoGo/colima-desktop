import { useState } from "react";
import { Radio, Copy, ExternalLink, Check, Settings2 } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { MdnsServiceEntry } from "../../types";

interface ContainerMdnsBadgeProps {
  service: MdnsServiceEntry | undefined;
  onConfigure: () => void;
}

export function ContainerMdnsBadge({
  service,
  onConfigure,
}: ContainerMdnsBadgeProps) {
  const [copied, setCopied] = useState(false);

  if (!service || !service.registered) {
    return (
      <button
        onClick={onConfigure}
        className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
        title="Configure mDNS"
      >
        <Radio className="h-3 w-3" />
        <span>{service ? "mDNS off" : "mDNS"}</span>
      </button>
    );
  }

  const mdnsAddress = `${service.hostname}.local:${service.port}`;
  const mdnsUrl = `http://${service.hostname}.local:${service.port}`;

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await navigator.clipboard.writeText(mdnsAddress);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleOpen = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await openUrl(mdnsUrl);
    } catch {
      window.open(mdnsUrl, "_blank");
    }
  };

  return (
    <span className="inline-flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400">
      <Radio className="h-3 w-3" />
      <span className="font-mono">{mdnsAddress}</span>
      {service.auto_registered && (
        <span className="text-[10px] text-muted-foreground bg-muted px-1 rounded">
          auto
        </span>
      )}
      <button
        onClick={handleCopy}
        className="p-0.5 rounded hover:bg-muted transition-colors"
        title="Copy address"
      >
        {copied ? (
          <Check className="h-3 w-3 text-emerald-600" />
        ) : (
          <Copy className="h-3 w-3" />
        )}
      </button>
      <button
        onClick={handleOpen}
        className="p-0.5 rounded hover:bg-muted transition-colors"
        title="Open in browser"
      >
        <ExternalLink className="h-3 w-3" />
      </button>
      <button
        onClick={onConfigure}
        className="p-0.5 rounded hover:bg-muted transition-colors"
        title="Configure mDNS"
      >
        <Settings2 className="h-3 w-3" />
      </button>
    </span>
  );
}
