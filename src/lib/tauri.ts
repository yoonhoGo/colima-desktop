import { invoke } from "@tauri-apps/api/core";
import type { Container, Image, ColimaStatus } from "../types";

export const api = {
  colimaStatus: () => invoke<ColimaStatus>("colima_status"),
  colimaStart: () => invoke<void>("colima_start"),
  colimaStop: () => invoke<void>("colima_stop"),
  colimaRestart: () => invoke<void>("colima_restart"),
  listContainers: () => invoke<Container[]>("list_containers"),
  containerStart: (id: string) => invoke<void>("container_start", { id }),
  containerStop: (id: string) => invoke<void>("container_stop", { id }),
  containerRestart: (id: string) => invoke<void>("container_restart", { id }),
  containerRemove: (id: string) => invoke<void>("container_remove", { id }),
  streamContainerLogs: (id: string) => invoke<void>("stream_container_logs", { id }),
  listImages: () => invoke<Image[]>("list_images"),
  pullImage: (name: string) => invoke<void>("pull_image", { name }),
  removeImage: (id: string) => invoke<void>("remove_image", { id }),
};
