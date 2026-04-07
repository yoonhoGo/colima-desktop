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
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (serviceType: string) => api.mdnsBrowse(serviceType),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mdns-state"] });
    },
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
