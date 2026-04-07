import { useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { setLiquidGlassEffect } from "tauri-plugin-liquid-glass-api";
import { MainLayout } from "./components/layout/MainLayout";
import "./App.css";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 1, refetchOnWindowFocus: true },
  },
});

export default function App() {
  useEffect(() => {
    setLiquidGlassEffect().catch(() => {
      // Liquid glass not supported on this platform — no-op
    });
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <MainLayout />
    </QueryClientProvider>
  );
}
