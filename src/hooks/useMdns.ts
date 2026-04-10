import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";
import type { MdnsConfig, ContainerMdnsOverride } from "../types";

export function useMdnsConfig() {
  return useQuery({
    queryKey: ["mdns-config"],
    queryFn: api.mdnsGetConfig,
  });
}

export function useMdnsSync(enabled: boolean) {
  return useQuery({
    queryKey: ["mdns-sync"],
    queryFn: api.mdnsSyncContainers,
    refetchInterval: enabled ? 5000 : false,
    enabled,
  });
}

export function useMdnsStatus() {
  return useQuery({
    queryKey: ["mdns-status"],
    queryFn: api.mdnsGetStatus,
  });
}

export function useMdnsSetConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (config: MdnsConfig) => api.mdnsSetConfig(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-config"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-sync"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-status"] });
    },
  });
}

export function useMdnsSetOverride() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      containerName,
      overrideConfig,
    }: {
      containerName: string;
      overrideConfig: ContainerMdnsOverride;
    }) => api.mdnsSetContainerOverride(containerName, overrideConfig),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-config"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-sync"] });
    },
  });
}

export function useMdnsRemoveOverride() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (containerName: string) =>
      api.mdnsRemoveContainerOverride(containerName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-config"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-sync"] });
    },
  });
}
