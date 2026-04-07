import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useMountSettings() {
  return useQuery({
    queryKey: ["mount-settings"],
    queryFn: api.getMountSettings,
    refetchInterval: 10000,
  });
}

export function useSaveMountSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: { mounts: { location: string; writable: boolean }[]; mountType: string; mountInotify: boolean }) =>
      api.saveMountSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["mount-settings"] });
    },
  });
}
