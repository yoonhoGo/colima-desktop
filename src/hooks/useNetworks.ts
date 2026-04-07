import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useNetworks() {
  return useQuery({
    queryKey: ["networks"],
    queryFn: api.listNetworks,
    refetchInterval: 10000,
  });
}

export function useCreateNetwork() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { name: string; driver?: string }) => api.createNetwork(params),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["networks"] }); },
  });
}

export function useRemoveNetwork() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.removeNetwork(id),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["networks"] }); },
  });
}

export function usePruneNetworks() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.pruneNetworks(),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["networks"] }); },
  });
}
