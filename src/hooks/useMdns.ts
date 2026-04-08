import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useMdnsState() {
  return useQuery({
    queryKey: ["mdns-state"],
    queryFn: api.mdnsGetState,
    refetchInterval: 5000,
  });
}

export function useMdnsEnable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: api.mdnsEnable,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
  });
}

export function useMdnsDisable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: api.mdnsDisable,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
  });
}

export function useMdnsBrowse() {
  return useMutation({
    mutationFn: (serviceType: string) => api.mdnsBrowse(serviceType),
  });
}

export function useMdnsRegisterContainer() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { containerName: string; port: number; serviceType?: string }) =>
      api.mdnsRegisterContainer(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
  });
}

export function useMdnsUnregisterService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { instanceName: string; serviceType: string }) =>
      api.mdnsUnregisterService(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
  });
}

export function useMdnsContainerConfigs() {
  return useQuery({
    queryKey: ["mdns-container-configs"],
    queryFn: api.mdnsGetContainerConfigs,
    refetchInterval: 5000,
  });
}

export function useMdnsSetContainerConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: {
      containerId: string;
      containerName: string;
      enabled: boolean;
      serviceType: string;
      port: number;
    }) => api.mdnsSetContainerConfig(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-container-configs"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
  });
}

export function useMdnsRemoveContainerConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (containerId: string) => api.mdnsRemoveContainerConfig(containerId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-container-configs"] });
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
  });
}

export function useMdnsSetAutoRegister() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (autoRegister: boolean) => api.mdnsSetAutoRegister(autoRegister),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
  });
}

export function useMdnsSyncContainers() {
  return useQuery({
    queryKey: ["mdns-sync"],
    queryFn: api.mdnsSyncContainers,
    refetchInterval: 5000,
  });
}
