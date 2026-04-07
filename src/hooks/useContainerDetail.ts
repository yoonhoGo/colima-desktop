import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useContainerDetail(id: string | null) {
  return useQuery({
    queryKey: ["container-detail", id],
    queryFn: () => api.containerInspect(id!),
    enabled: !!id,
  });
}

export function useContainerStats(id: string | null) {
  return useQuery({
    queryKey: ["container-stats", id],
    queryFn: () => api.containerStats(id!),
    enabled: !!id,
    refetchInterval: 5000,
  });
}
