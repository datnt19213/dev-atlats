import type { FC } from "react";

export type Page =
  | "dashboard"
  | "explorer"
  | "scanner"
  | "graphs"
  | "documentation"
  | "diagrams"
  | "exports"
  | "settings";

type IconProps = { size?: number };

const DashboardIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="18" height="18" /></svg>
);

const FolderIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" /></svg>
);

const ScanIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 2l-9 9-4-4-9 9" /><path d="M21 15l-9-9" /></svg>
);

const GraphIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="6" cy="6" r="3" /><circle cx="18" cy="6" r="3" /><circle cx="6" cy="18" r="3" /><path d="M8.7 8.7 15.3 15.3" /><path d="M15.3 8.7 8.7 15.3" /><path d="M6 9v6" /><path d="M18 9v6" /></svg>
);

const FileTextIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /></svg>
);

const GitBranchIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="6" y1="3" x2="6" y2="15" /><line x1="18" y1="3" y2="15" /><path d="M6 9a6 6 0 0 0 12 0" /></svg>
);

const ArchiveIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="21 8 21 21 3 21 3 8" /><rect x="1" y="3" width="22" height="5" /></svg>
);

const SettingsIcon: FC<IconProps> = ({ size = 17 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3" /><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" /></svg>
);

export const navigation: Array<{ page: Page; label: string; Icon: FC<IconProps> }> = [
  { page: "dashboard", label: "Dashboard", Icon: DashboardIcon },
  { page: "explorer", label: "Explorer", Icon: FolderIcon },
  { page: "scanner", label: "Scanner", Icon: ScanIcon },
  { page: "graphs", label: "Graph", Icon: GraphIcon },
  { page: "documentation", label: "Documentation", Icon: FileTextIcon },
  { page: "diagrams", label: "Diagrams", Icon: GitBranchIcon },
  { page: "exports", label: "Exports", Icon: ArchiveIcon },
  { page: "settings", label: "Settings", Icon: SettingsIcon },
];

export function pageTitle(page: Page): string {
  const titles: Record<Page, string> = {
    dashboard: "Dashboard",
    explorer: "Explorer",
    scanner: "Scanner",
    graphs: "Graph",
    documentation: "Documentation",
    diagrams: "Diagrams",
    exports: "Exports",
    settings: "Settings",
  };
  return titles[page] ?? "DevAtlas";
}
