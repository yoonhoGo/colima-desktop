import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useNetworkSettings() {
  return useQuery({
    queryKey: ["network-settings"],
    queryFn: api.getNetworkSettings,
    refetchInterval: 10000,
  });
}

export function useSaveNetworkSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: {
      dns: string[];
      dnsHosts: { hostname: string; ip: string }[];
      networkAddress: boolean;
      networkMode: string;
      gatewayAddress: string;
      networkInterface: string;
      portForwarder: string;
    }) => api.saveNetworkSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["network-settings"] });
    },
  });
}
