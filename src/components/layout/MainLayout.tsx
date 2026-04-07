import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { ContainerList } from "../containers/ContainerList";
import { ImageList } from "../images/ImageList";
import { VolumeList } from "../volumes/VolumeList";
import { NetworkList } from "../networks/NetworkList";
import { VmSettings } from "../settings/VmSettings";
import { MountSettings } from "../settings/MountSettings";
import { NetworkSettingsPanel } from "../settings/NetworkSettingsPanel";
import { DockerSettingsPanel } from "../settings/DockerSettingsPanel";
import { UpdatePanel } from "../settings/UpdatePanel";

type Page = "containers" | "images" | "volumes" | "networks" | "settings";
type SettingsTab = "vm" | "mounts" | "network" | "docker" | "update";

export function MainLayout() {
  const [activePage, setActivePage] = useState<Page>("containers");
  const [settingsTab, setSettingsTab] = useState<SettingsTab>("vm");

  return (
    <div className="flex h-screen">
      <Sidebar activePage={activePage} onPageChange={setActivePage} />
      <main className="flex-1 overflow-auto p-4">
        {activePage === "containers" && <ContainerList />}
        {activePage === "images" && <ImageList />}
        {activePage === "volumes" && <VolumeList />}
        {activePage === "networks" && <NetworkList />}
        {activePage === "settings" && (
          <div className="space-y-4">
            <div className="mx-auto max-w-lg flex gap-1 rounded-lg bg-muted p-1">
              <button
                onClick={() => setSettingsTab("vm")}
                className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
                  settingsTab === "vm"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                VM
              </button>
              <button
                onClick={() => setSettingsTab("mounts")}
                className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
                  settingsTab === "mounts"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Mounts
              </button>
              <button
                onClick={() => setSettingsTab("network")}
                className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
                  settingsTab === "network"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Network
              </button>
              <button
                onClick={() => setSettingsTab("docker")}
                className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
                  settingsTab === "docker"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Docker
              </button>
              <button
                onClick={() => setSettingsTab("update")}
                className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
                  settingsTab === "update"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Update
              </button>
            </div>
            {settingsTab === "vm" && <VmSettings />}
            {settingsTab === "mounts" && <MountSettings />}
            {settingsTab === "network" && <NetworkSettingsPanel />}
            {settingsTab === "docker" && <DockerSettingsPanel />}
            {settingsTab === "update" && <UpdatePanel />}
          </div>
        )}
      </main>
    </div>
  );
}
