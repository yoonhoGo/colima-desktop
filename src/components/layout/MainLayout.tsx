import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { ContainerList } from "../containers/ContainerList";
import { ImageList } from "../images/ImageList";

type Page = "containers" | "images";

export function MainLayout() {
  const [activePage, setActivePage] = useState<Page>("containers");

  return (
    <div className="flex h-screen">
      <Sidebar activePage={activePage} onPageChange={setActivePage} />
      <main className="flex-1 overflow-auto p-4">
        {activePage === "containers" && <ContainerList />}
        {activePage === "images" && <ImageList />}
      </main>
    </div>
  );
}
