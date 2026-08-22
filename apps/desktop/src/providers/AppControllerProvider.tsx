import type { ReactNode } from "react";
import { createContext, useContext } from "react";

import { useAppController } from "@/handlers/use-app-controller";

const AppControllerContext = createContext<ReturnType<typeof useAppController> | null>(null);

export function AppControllerProvider({ children }: { children: ReactNode }) {
  const controller = useAppController();

  return (
    <AppControllerContext.Provider value={controller}>
      {children}
    </AppControllerContext.Provider>
  );
}

export function useAppControllerContext() {
  const controller = useContext(AppControllerContext);

  if (!controller) {
    throw new Error("useAppControllerContext must be used inside AppControllerProvider");
  }

  return controller;
}
