import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useVolumes() {
  return useQuery({
    queryKey: ["volumes"],
    queryFn: api.listVolumes,
    refetchInterval: 10000,
  });
}

export function useCreateVolume() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { name: string; driver?: string }) => api.createVolume(params),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["volumes"] }); },
  });
}

export function useRemoveVolume() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => api.removeVolume(name),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["volumes"] }); },
  });
}

export function usePruneVolumes() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.pruneVolumes(),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["volumes"] }); },
  });
}
