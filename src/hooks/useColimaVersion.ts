import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useColimaVersion() {
  return useQuery({
    queryKey: ["colima-version"],
    queryFn: api.getColimaVersion,
  });
}

export function useVersionCheck() {
  return useQuery({
    queryKey: ["version-check"],
    queryFn: api.checkLatestVersion,
    staleTime: 60000, // check at most once per minute
  });
}

export function useUpdateRuntime() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: api.updateColimaRuntime,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["colima-version"] });
      queryClient.invalidateQueries({ queryKey: ["version-check"] });
    },
  });
}
