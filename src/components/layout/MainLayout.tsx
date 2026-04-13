import { useState } from "react";
import { Sidebar } from "./Sidebar";
import type { Page, ComposeFilter } from "./Sidebar";
import { ContainerList } from "../containers/ContainerList";
import { ImageList } from "../images/ImageList";
import { VolumeList } from "../volumes/VolumeList";
import { NetworkList } from "../networks/NetworkList";
import { VmSettings } from "../settings/VmSettings";
import { MountSettings } from "../settings/MountSettings";
import { NetworkSettingsPanel } from "../settings/NetworkSettingsPanel";
import { DockerSettingsPanel } from "../settings/DockerSettingsPanel";
import { UpdatePanel } from "../settings/UpdatePanel";
import { AppearanceSettings } from "../settings/AppearanceSettings";
import { TerminalSettings } from "../settings/TerminalSettings";
import { ContainerDomainsSettings } from "../settings/ContainerDomainsSettings";
import { EnvironmentPage } from "../environment/EnvironmentPage";

export function MainLayout() {
  const [activePage, setActivePage] = useState<Page>("containers");
  const [composeFilter, setComposeFilter] = useState<ComposeFilter>(null);

  return (
    <div className="relative z-10 flex h-screen">
      <Sidebar
        activePage={activePage}
        onPageChange={setActivePage}
        composeFilter={composeFilter}
        onComposeFilter={setComposeFilter}
      />
      <main className="flex-1 min-w-0 overflow-auto p-4">
        {activePage === "containers" && (
          <ContainerList composeFilter={composeFilter} />
        )}
        {activePage === "images" && <ImageList />}
        {activePage === "volumes" && <VolumeList />}
        {activePage === "networks" && <NetworkList />}
        {activePage === "environment" && <EnvironmentPage />}
        {activePage === "settings/vm" && <VmSettings />}
        {activePage === "settings/mounts" && <MountSettings />}
        {activePage === "settings/network" && <NetworkSettingsPanel />}
        {activePage === "settings/docker" && <DockerSettingsPanel />}
        {activePage === "settings/domains" && <ContainerDomainsSettings />}
        {activePage === "settings/terminal" && <TerminalSettings />}
        {activePage === "settings/update" && <UpdatePanel />}
        {activePage === "settings/appearance" && <AppearanceSettings />}
      </main>
    </div>
  );
}
