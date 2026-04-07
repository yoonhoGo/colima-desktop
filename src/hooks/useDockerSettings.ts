import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useDockerSettings() {
  return useQuery({
    queryKey: ["docker-settings"],
    queryFn: api.getDockerSettings,
    refetchInterval: 10000,
  });
}

export function useSaveDockerSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: { insecureRegistries: string[]; registryMirrors: string[] }) =>
      api.saveDockerSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["docker-settings"] });
    },
  });
}
