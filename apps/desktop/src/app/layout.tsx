import { useEffect, useState, type ReactElement, type ReactNode, useRef, useCallback } from "react";
import { Boxes, Copy, ExternalLink, X } from "lucide-react";
import { gsap } from "gsap";
import { useTheme } from "next-themes";

import { DashboardPage } from "@/features/dashboard";
import { DiagramsPage } from "@/features/diagrams";
import { DocumentationPage } from "@/features/documentation";
import { ExportsPage } from "@/features/exports";
import { GraphsPage } from "@/features/graphs";
import { ExplorerPage } from "@/features/explorer";
import { ScannerPage } from "@/features/scanner";
import { SettingsPage } from "@/features/settings";
import { Button } from "@/components/ui/button";
import { Sidebar } from "@/components/ui/sidebar";
import { navigation, pageTitle, type Page } from "@/handlers/navigation";
import { DiagramPreview } from "@/components/ui/diagram-preview";
import { encode } from "plantuml-encoder";
import mermaid from "mermaid";
import { open } from "@tauri-apps/plugin-shell";
import { useAppControllerContext } from "@/providers/AppControllerProvider";
import { cn } from "@/lib/utils";

export function AppLayout({ children }: { children: ReactNode }): ReactElement {
  const controller = useAppControllerContext();
  const { setTheme } = useTheme();
  const activeIndicatorRef = useRef<HTMLDivElement | null>(null);
  const collapsed = controller.uiPreferences.sidebarCollapsed;
  const backdropClass = getBackdropClass(controller.uiPreferences.backdropMode);
  const activeIndex = navigation.findIndex((item) => item.page === controller.page);

  useEffect(() => {
    setTheme(controller.uiPreferences.themeMode);
  }, [controller.uiPreferences.themeMode, setTheme]);

  useEffect(() => {
    if (!activeIndicatorRef.current || activeIndex < 0) return;
    gsap.to(activeIndicatorRef.current, {
      y: activeIndex * 52,
      duration: 0.28,
      ease: "power2.out",
    });
  }, [activeIndex]);

  const preview = controller.modalContent;
  const [copied, setCopied] = useState(false);
  const { onCloseModal } = controller;

  useEffect(() => {
    if (!preview) return;
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") onCloseModal();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [preview, onCloseModal]);
  const handleOpenInBrowser = useCallback(async () => {
    if (!preview) return;
    const { content, format } = preview;
    const normalizedFormat = (format ?? "text").toLowerCase();

    let url: string;
    if (normalizedFormat === "plantuml") {
      url = "https://www.plantuml.com/plantuml/svg/" + encode(content);
    } else if (normalizedFormat === "mermaid") {
      const id = "mermaid-open-" + Math.random().toString(36).slice(2);
      const tempDiv = document.createElement("div");
      tempDiv.style.position = "absolute";
      tempDiv.style.left = "-9999px";
      document.body.appendChild(tempDiv);
      try {
        const { svg } = await mermaid.render(id, content);
        const blob = new Blob([svg], { type: "image/svg+xml" });
        url = URL.createObjectURL(blob);
      } catch (err) {
        console.error("Failed to render Mermaid diagram:", err);
        return;
      } finally {
        document.body.removeChild(tempDiv);
      }
    } else if (normalizedFormat === "svg") {
      const blob = new Blob([content], { type: "image/svg+xml" });
      url = URL.createObjectURL(blob);
    } else {
      url = "data:text/plain," + encodeURIComponent(content);
    }

    open(url).catch((err) => {
      console.error("Failed to open URL in browser:", err);
      alert("Could not open the diagram in your browser. Please check your browser settings.");
    });
  }, [preview]);

  return (
    <div
      className={[
        "grid h-screen overflow-hidden transition-[grid-template-columns] duration-500 ease-out",
        collapsed ? "[grid-template-columns:64px_minmax(0,1fr)]" : "[grid-template-columns:280px_minmax(0,1fr)]",
        backdropClass,
      ].join(" ")}
    >
      <Sidebar className="relative flex h-screen min-h-0 flex-col overflow-hidden border-r border-sidebar-border px-2 py-6">
        <button
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          className="group/sidebar-toggle outline-none absolute right-0 top-0 z-30 h-full w-3 cursor-pointer bg-transparent opacity-0 transition-opacity duration-200 hover:opacity-100"
          type="button"
          onClick={() => controller.updateUiPreferences({ sidebarCollapsed: !collapsed })}
        >
          <span className="absolute right-0 top-0 h-full w-1 bg-foreground shadow-sm transition-colors group-hover/sidebar-toggle:bg-primary" />
        </button>
        <div className={cn("relative mb-7 flex h-14  items-center gap-3 rounded-none bg-card/80", collapsed ? "justify-center" : "justify-start w-[calc(100%-12px)] mx-auto")}>
          <div className="grid h-9 w-9 shrink-0 place-items-center rounded-none bg-primary text-primary-foreground">
            <Boxes size={17} />
          </div>
          <div className={["grid min-w-0 gap-0.5", collapsed ? "hidden" : ""].join(" ")}>
            <strong className="text-[15px] font-semibold leading-5">DevAtlas</strong>
            <span className="text-xs leading-4 text-muted-foreground">Repository intelligence</span>
          </div>
        </div>

        <div className={["relative mb-4 px-2 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground", collapsed ? "hidden" : ""].join(" ")}>
          Workspace
        </div>

        <nav className={cn("relative grid gap-2 overflow-hidden py-1", collapsed ? "px-0" : "px-1.5")}>
          <div ref={activeIndicatorRef} className={cn("pointer-events-none absolute top-1 h-11 rounded-none bg-primary/30", collapsed ? "left-0 w-full" : "left-1.5 w-[calc(100%-12px)]")} aria-hidden="true" />
          {navigation.map((item, index) => (
            <NavItem
              collapsed={collapsed}
              isActive={controller.page === item.page}
              item={item}
              key={item.page}
              onPageChange={() => controller.setPage(item.page)}
              index={index}
            />
          ))}
        </nav>

        <div className="relative mx-1.5 mt-auto grid min-h-16 place-items-center rounded-none bg-card/80 p-2">
          {collapsed ? (
            <Boxes size={17} className="text-primary" aria-hidden="true" />
          ) : (
            <div className="grid w-full gap-2">
              <span className="text-xs font-medium uppercase tracking-[0.08em] text-muted-foreground">
                Current repository
              </span>
              <strong className="truncate text-sm font-semibold leading-5">
                {controller.repository?.name ?? "None opened"}
              </strong>
              <p className="m-0 text-xs leading-5 text-muted-foreground">
                {controller.scan ? `${controller.scan.files} files / ${controller.scan.folders} folders` : "Run scanner to build project context."}
              </p>
            </div>
          )}
        </div>
      </Sidebar>

      <main className="relative h-screen min-w-0 overflow-y-auto [scrollbar-width:thin] [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:rounded-none [&::-webkit-scrollbar-thumb]:bg-border/60 px-4 py-5 sm:px-6 sm:py-6 lg:px-8 xl:px-10 xl:py-8">
        <div className="relative mx-auto grid w-full max-w-[1440px] gap-6 sm:gap-8">
          <header className="flex min-h-20 flex-col gap-5 border-b border-border bg-card/90 p-5 backdrop-blur-xl sm:flex-row sm:items-start sm:justify-between sm:gap-6">
            <div className="grid min-w-0 gap-1.5">
              <h1 className="m-0 text-3xl font-bold leading-tight tracking-tight text-foreground">{pageTitle(controller.page)}</h1>
              <p className="m-0 truncate text-sm leading-6 text-muted-foreground">
                {controller.repository ? controller.repository.path : "Open a repository to start analysis."}
              </p>
            </div>
          </header>
          <div>{getPageContent(controller.page)}</div>
        </div>
      </main>

      {preview && (
        <div
          className="fixed inset-0 z-[90] grid place-items-center bg-background/95 p-4 backdrop-blur-xl"
          onClick={onCloseModal}
          role="presentation"
        >
          <div
            className="grid max-h-[90vh] w-full max-w-5xl overflow-hidden rounded-none border bg-card shadow-2xl"
            onClick={(event) => event.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label={preview.title}
          >
            <header className="flex items-start justify-between gap-3 border-b border-border p-5">
              <div className="grid min-w-0 gap-1">
                <h2 className="m-0 text-lg font-semibold leading-6 text-foreground">{preview.title}</h2>
                {preview.subtitle && <p className="m-0 text-sm leading-5 text-muted-foreground">{preview.subtitle}</p>}
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <Button
                  type="button"
                  variant="secondary"
                  size="sm"
                  onClick={handleOpenInBrowser}
                >
                  <ExternalLink size={14} />
                  Open in browser
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  size="sm"
                  onClick={() => {
                    navigator.clipboard.writeText(preview.content);
                    setCopied(true);
                    setTimeout(() => setCopied(false), 1500);
                  }}
                >
                  <Copy size={14} />
                  {copied ? "Copied" : "Copy"}
                </Button>
                <button
                  type="button"
                  aria-label="Close"
                  onClick={onCloseModal}
                  className="grid h-9 w-9 place-items-center rounded-none text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                >
                  <X size={18} />
                </button>
              </div>
            </header>
            <div className="overflow-auto p-5">
              <DiagramPreview content={preview.content} format={preview.format ?? "text"} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function NavItem({
  collapsed,
  isActive,
  index,
  item,
  onPageChange,
}: {
  collapsed: boolean;
  index: number;
  isActive: boolean;
  item: { page: Page; label: string; Icon: React.ElementType };
  onPageChange: () => void;
}): ReactElement {
  const buttonRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    if (!buttonRef.current) return;
    gsap.fromTo(buttonRef.current, { x: 0, opacity: 0.72 }, { x: 0, opacity: 1, duration: 0.24, ease: "power2.out" });
  }, [index]);

  return (
    <Button
      ref={buttonRef}
      aria-current={isActive ? "page" : undefined}
      aria-label={item.label}
      className={["group relative z-10 grid h-11 w-full grid-cols-[32px_minmax(0,1fr)] items-center overflow-hidden rounded-none bg-transparent px-3 text-left text-foreground hover:bg-transparent", collapsed ? "gap-0" : "gap-3"].join(" ")}
      type="button"
      onClick={onPageChange}
      onMouseEnter={() => {
        if (!buttonRef.current || collapsed) return;
        gsap.to(buttonRef.current, { x: 4, duration: 0.16, ease: "power2.out" });
      }}
      onMouseLeave={() => {
        if (!buttonRef.current) return;
        gsap.to(buttonRef.current, { x: 0, duration: 0.16, ease: "power2.out" });
      }}
    >
      <span className="grid h-8 w-8 place-items-center rounded-none text-primary transition-colors group-hover:text-primary">
        <item.Icon size={17} />
      </span>
      <span className={["min-w-0 truncate text-sm font-medium leading-5 transition-opacity", collapsed ? "translate-x-2 opacity-0" : "translate-x-0 opacity-100"].join(" ")}>{item.label}</span>
    </Button>
  );
}

function getPageContent(page: Page): ReactElement {
  switch (page) {
    case "dashboard":
      return <DashboardPage />;
    case "explorer":
      return <ExplorerPage />;
    case "scanner":
      return <ScannerPage />;
    case "graphs":
      return <GraphsPage />;
    case "documentation":
      return <DocumentationPage />;
    case "diagrams":
      return <DiagramsPage />;
    case "exports":
      return <ExportsPage />;
    case "settings":
      return <SettingsPage />;
  }
}

function getBackdropClass(mode: "aurora" | "mesh" | "plain"): string {
  if (mode === "mesh") return "bg-muted";
  if (mode === "plain") return "bg-background";
  return "bg-gradient-to-br from-background via-secondary to-muted";
}

