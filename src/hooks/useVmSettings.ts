import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useVmSettings() {
  return useQuery({
    queryKey: ["vm-settings"],
    queryFn: api.getVmSettings,
    refetchInterval: 10000,
  });
}

export function useHostInfo() {
  return useQuery({
    queryKey: ["host-info"],
    queryFn: api.getHostInfo,
    staleTime: Infinity,
  });
}

export function useApplyVmSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: {
      cpus: number;
      memoryGib: number;
      diskGib: number;
      runtime: string;
      networkAddress: string;
    }) => api.applyVmSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["vm-settings"] });
      queryClient.invalidateQueries({ queryKey: ["colima-status"] });
    },
  });
}
