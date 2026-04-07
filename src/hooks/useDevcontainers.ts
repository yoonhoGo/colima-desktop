import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useDevcontainerCliCheck() {
  return useQuery({
    queryKey: ["devcontainer-cli-check"],
    queryFn: api.checkDevcontainerCli,
    staleTime: 60_000,
  });
}

export function useDevcontainerProjects() {
  return useQuery({
    queryKey: ["devcontainer-projects"],
    queryFn: api.listDevcontainerProjects,
    refetchInterval: 3000,
  });
}

export function useAddDevcontainerProject() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (workspacePath: string) => api.addDevcontainerProject(workspacePath),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devcontainer-projects"] });
    },
  });
}

export function useRemoveDevcontainerProject() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, removeContainer }: { id: string; removeContainer: boolean }) =>
      api.removeDevcontainerProject(id, removeContainer),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devcontainer-projects"] });
    },
  });
}

export function useDevcontainerAction() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      workspacePath,
      action,
    }: {
      workspacePath: string;
      action: "up" | "build" | "stop";
    }) => {
      switch (action) {
        case "up":
          return api.devcontainerUp(workspacePath);
        case "build":
          return api.devcontainerBuild(workspacePath);
        case "stop":
          return api.devcontainerStop(workspacePath);
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devcontainer-projects"] });
    },
  });
}

export function useDevcontainerConfig(workspacePath: string) {
  return useQuery({
    queryKey: ["devcontainer-config", workspacePath],
    queryFn: () => api.devcontainerReadConfig(workspacePath),
    enabled: !!workspacePath,
    staleTime: 30_000,
  });
}
