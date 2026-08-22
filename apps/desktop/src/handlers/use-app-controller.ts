import { useEffect, useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";

import {
  buildGraph as buildGraphCommand,
  detectTechnologies as detectTechnologiesCommand,
  exportPackage as exportPackageCommand,
  generateDiagrams as generateDiagramsCommand,
  generateDocs as generateDocsCommand,
  getGraph as getGraphCommand,
  getCommandErrorMessage,
  isTauriRuntime,
  listRepositoryFiles as listRepositoryFilesCommand,
  openRepository as openRepositoryCommand,
  scanRepository as scanRepositoryCommand,
} from "@/services/commands";
import { useAppStore } from "../stores/app-store";
import { defaultUiPreferences, scanMaxFilesFromPreferences, type UiPreferences } from "./preferences";
import type { Page } from "./navigation";
import type { ChatTurn, ToastMessage, ModalContent } from "./types";
import type { ExportPackage, GeneratedDocument, DiagramResult, GraphEdge, GraphNode, RepositoryFile, RepositorySummary, ScanResult } from "@/types/contracts";

export function useAppController() {
  const queryClient = useQueryClient();
  const [page, setPageState] = useState<Page>(getRoutePage());
  const [repositoryPath, setRepositoryPath] = useState("");
  const [outputPath, setOutputPath] = useState("");
  const [explorerFiles, setExplorerFiles] = useState<RepositoryFile[]>([]);
  const [analysisScopePaths, setAnalysisScopePaths] = useState<string[]>([]);
  const [scopeMode, setScopeMode] = useState<"folders" | "files" | "folders-extra">("folders");
  const [analyzeConfirmOpen, setAnalyzeConfirmOpen] = useState(false);
  const [chatQuestion, setChatQuestion] = useState("");
  const [chatTurns, setChatTurns] = useState<ChatTurn[]>([]);
  const [toasts, setToasts] = useState<ToastMessage[]>([]);
  const [modalContent, setModalContent] = useState<ModalContent | undefined>();
  const [uiPreferences, setUiPreferences] = useState(loadUiPreferences());
  const [processingAction, setProcessingAction] = useState<"select" | "open" | "detect" | "analyze" | null>(null);

  const {
    repository,
    scan,
    graph,
    graphNodes,
    graphEdges,
    documents,
    diagrams,
    exportPackage: exportedPackage,
    status,
    error,
    setRepository,
    setScan,
    setGraph,
    setGraphNodes,
    setGraphEdges,
    setDocuments,
    setDiagrams,
    setExportPackage,
    setStatus,
    setError,
  } = useAppStore();

  const nativeRuntimeAvailable = isTauriRuntime();

  useEffect(() => {
    if (typeof window.localStorage?.setItem === "function") {
      window.localStorage.setItem("devatlas-ui-preferences", JSON.stringify(uiPreferences));
    }
  }, [uiPreferences]);

  useEffect(() => {
    if (toasts.length === 0) return;
    const timeoutId = window.setTimeout(() => {
      setToasts((currentToasts) => currentToasts.slice(1));
    }, 4200);
    return () => window.clearTimeout(timeoutId);
  }, [toasts]);

  useEffect(() => {
    function handleRouteChange() {
      setPageState(getRoutePage());
    }
    window.addEventListener("hashchange", handleRouteChange);
    return () => window.removeEventListener("hashchange", handleRouteChange);
  }, []);

  useEffect(() => {
    setExplorerFiles(scan?.repositoryFiles ?? []);
  }, [scan?.repositoryFiles]);

  function setPage(nextPage: Page): void {
    window.location.hash = `/${nextPage}`;
    setPageState(nextPage);
  }

  function updateUiPreferences(nextPreferences: Partial<UiPreferences>) {
    setUiPreferences((current) => ({ ...current, ...nextPreferences }));
  }

  function notify(message: Omit<ToastMessage, "id">) {
    setToasts((current) => [...current.slice(-2), { ...message, id: `${Date.now()}-${current.length}` }]);
  }

  function clearError() {
    if (error) setError(undefined);
  }

  function getScanMaxFiles() {
    return scanMaxFilesFromPreferences(uiPreferences);
  }

  async function selectAndOpenRepository(): Promise<void> {
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to open this repository.", tone: "warning" });
      return;
    }
    const typedPath = repositoryPath.trim();
    if (typedPath.length > 0) {
      await openRepositoryPath(typedPath);
      return;
    }
    setProcessingAction("open");
    try {
      const selected = await open({ directory: true, multiple: false, title: "Select repository" });
      if (typeof selected !== "string") {
        setProcessingAction(null);
        return;
      }
      await openRepositoryPath(selected);
    } finally {
      setProcessingAction(null);
    }
  }

  async function openRepositoryPath(path?: string): Promise<void> {
    const trimmedPath: string = (path ?? repositoryPath).trim();
    if (trimmedPath.length === 0) {
      notify({ title: "Repository path required", tone: "error" });
      return;
    }
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to open this repository.", tone: "warning" });
      return;
    }
    setStatus("working");
    clearError();
    try {
      setProcessingAction("open");
      const openedRepository = await openRepositoryCommand(trimmedPath);
      const isSameRepository = repository != null && repository.path === openedRepository.path;
      setRepositoryPath(trimmedPath);
      setRepository(openedRepository);
      if (!isSameRepository) {
        setScan(undefined);
        setGraph(undefined);
        setGraphNodes([]);
        setGraphEdges([]);
        setDocuments([]);
        setDiagrams([]);
        setExportPackage(undefined);
      }
      setStatus("ready");
      setProcessingAction(null);
      notify({ title: "Repository opened", detail: openedRepository.path, tone: "success" });
    } catch (commandError) {
      setStatus("error");
      setError(getCommandErrorMessage(commandError));
      setProcessingAction(null);
      notify({ title: "Failed to open repository", detail: getCommandErrorMessage(commandError), tone: "error" });
    }
  }

  async function detectTechnologies(): Promise<void> {
    if (!repository) {
      notify({ title: "Repository required", detail: "Open a repository before detecting technologies.", tone: "warning" });
      return;
    }
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to detect technologies.", tone: "warning" });
      return;
    }
    setStatus("working");
    clearError();
    try {
      setProcessingAction("detect");
      const technologies = await detectTechnologiesCommand();
      const files = await listRepositoryFilesCommand();
      const nextScan: ScanResult = {
        scanId: `scan-${Date.now()}`,
        repositoryId: repository.repositoryId,
        files: files.length,
        folders: countFolders(files),
        durationMs: 0,
        technologies,
        repositoryFiles: files,
      };
      setScan(nextScan);
      setExplorerFiles(files);
      setStatus("ready");
      setProcessingAction(null);
      notify({ title: "Technologies detected", detail: `${technologies.length} technologies`, tone: "success" });
    } catch (commandError) {
      setStatus("error");
      setError(getCommandErrorMessage(commandError));
      setProcessingAction(null);
      notify({ title: "Failed to detect technologies", detail: getCommandErrorMessage(commandError), tone: "error" });
    }
  }

  async function analyzeRepository(): Promise<void> {
    if (!repository) {
      notify({ title: "Repository required", detail: "Open a repository before analysis.", tone: "warning" });
      return;
    }
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to analyze this repository.", tone: "warning" });
      return;
    }
    setStatus("working");
    clearError();
    try {
      setProcessingAction("analyze");
      const scanResult = await scanRepositoryCommand(getScanMaxFiles(), getEffectiveScopePaths());
      const files = await listRepositoryFilesCommand();
      const technologies = await detectTechnologiesCommand();
      const graphSummary = await buildGraphCommand();
      const knowledgeGraph = await getGraphCommand();

      const nextScan: ScanResult = {
        ...scanResult,
        technologies,
        repositoryFiles: files,
      };

      setScan(nextScan);
      setExplorerFiles(files);
      setGraph({
        nodeCount: graphSummary.nodeCount,
        edgeCount: graphSummary.edgeCount,
      });
      setGraphNodes(knowledgeGraph.nodes);
      setGraphEdges(knowledgeGraph.edges);
      setStatus("ready");
      setProcessingAction(null);
      notify({ title: "Analysis complete", detail: `${files.length} files / ${graphSummary.nodeCount} nodes`, tone: "success" });
    } catch (commandError) {
      setStatus("error");
      setError(getCommandErrorMessage(commandError));
      setProcessingAction(null);
      notify({ title: "Failed to analyze repository", detail: getCommandErrorMessage(commandError), tone: "error" });
    }
  }

  async function refreshFiles(): Promise<void> {
    if (!repository) {
      notify({ title: "Repository required", detail: "Open a repository before refreshing files.", tone: "warning" });
      return;
    }
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to refresh files.", tone: "warning" });
      return;
    }
    setStatus("working");
    clearError();
    try {
      const files = await listRepositoryFilesCommand();
      setExplorerFiles(files);
      if (scan) {
        setScan({ ...scan, files: files.length, repositoryFiles: files });
      }
      setStatus("ready");
      notify({ title: "Explorer refreshed", detail: `${files.length} files`, tone: "success" });
    } catch (commandError) {
      setStatus("error");
      setError(getCommandErrorMessage(commandError));
      notify({ title: "Failed to refresh files", detail: getCommandErrorMessage(commandError), tone: "error" });
    }
  }

  function isFileUnderSelectedFolder(path: string): boolean {
    return analysisScopePaths.some((scopePath) => {
      const isFolder = !explorerFiles.some((file) => file.path === scopePath);
      return isFolder && (path === scopePath || path.startsWith(scopePath + "/"));
    });
  }

  function isScopeActive(path: string, kind: "folder" | "file"): boolean {
    if (kind === "folder") return scopeMode !== "files" && analysisScopePaths.includes(path);
    if (scopeMode === "folders") return false;
    if (scopeMode === "folders-extra" && isFileUnderSelectedFolder(path)) return false;
    return analysisScopePaths.includes(path);
  }

  function isScopeDisabled(path: string, kind: "folder" | "file"): boolean {
    if (kind === "folder") return scopeMode === "files";
    if (kind === "file") return scopeMode === "folders" || (scopeMode === "folders-extra" && isFileUnderSelectedFolder(path));
    return false;
  }

  function toggleScopeSelection(path: string, kind: "folder" | "file"): void {
    if (isScopeDisabled(path, kind)) return;
    setAnalysisScopePaths((current) => current.includes(path) ? current.filter((item) => item !== path) : [...current, path]);
  }

  function clearScope(): void {
    setAnalysisScopePaths([]);
  }

  function selectAllScope(): void {
    if (scopeMode === "files") {
      setAnalysisScopePaths(explorerFiles.map((file) => file.path));
      return;
    }
    const folderPaths = Array.from(new Set(explorerFiles.map((file) => getScopePath(file.path))));
    setAnalysisScopePaths((current) => Array.from(new Set([...current, ...folderPaths])));
  }

  function getEffectiveScopePaths(): string[] {
    if (scopeMode === "folders") return analysisScopePaths.filter((path) => !explorerFiles.some((file) => file.path === path));
    if (scopeMode === "files") return analysisScopePaths.filter((path) => explorerFiles.some((file) => file.path === path));
    return analysisScopePaths;
  }

  function getAnalysisScopeFiles(): RepositoryFile[] {
    const effective = getEffectiveScopePaths();
    if (effective.length === 0) return explorerFiles;
    return explorerFiles.filter((file) => effective.some((scopePath) => file.path === scopePath || file.path.startsWith(scopePath + "/")));
  }
  async function generateDocuments(): Promise<void> {
    if (!repository) {
      notify({ title: "Repository required", detail: "Open a repository before generating documentation.", tone: "warning" });
      return;
    }
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to generate documentation.", tone: "warning" });
      return;
    }
    setStatus("working");
    clearError();
    try {
      const generatedDocuments: GeneratedDocument[] = await generateDocsCommand();
      setDocuments(generatedDocuments);
      setStatus("ready");
      notify({ title: "Documentation generated", detail: `${generatedDocuments.length} documents`, tone: "success" });
    } catch (commandError) {
      setStatus("error");
      setError(getCommandErrorMessage(commandError));
      notify({ title: "Failed to generate documentation", detail: getCommandErrorMessage(commandError), tone: "error" });
    }
  }

  async function generateDiagrams(): Promise<void> {
    if (!repository) {
      notify({ title: "Repository required", detail: "Open a repository before generating diagrams.", tone: "warning" });
      return;
    }
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to generate diagrams.", tone: "warning" });
      return;
    }
    setStatus("working");
    clearError();
    try {
      const generatedDiagrams: DiagramResult[] = await generateDiagramsCommand();
      setDiagrams(generatedDiagrams);
      setStatus("ready");
      notify({ title: "Diagrams generated", detail: `${generatedDiagrams.length} diagrams`, tone: "success" });
    } catch (commandError) {
      setStatus("error");
      setError(getCommandErrorMessage(commandError));
      notify({ title: "Failed to generate diagrams", detail: getCommandErrorMessage(commandError), tone: "error" });
    }
  }

  async function selectOutputFolder(): Promise<void> {
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to select an export folder.", tone: "warning" });
      return;
    }
    const selected = await open({ directory: true, multiple: false, title: "Select export folder" });
    if (typeof selected === "string") {
      setOutputPath(selected);
      notify({ title: "Output folder selected", detail: selected, tone: "success" });
    }
  }

  async function exportKnowledgePackage(): Promise<void> {
    if (outputPath.trim().length === 0) {
      notify({ title: "Output folder required", tone: "error" });
      return;
    }
    if (!nativeRuntimeAvailable) {
      notify({ title: "Desktop bridge unavailable", detail: "Open DevAtlas in the desktop app to export the package.", tone: "warning" });
      return;
    }
    setStatus("working");
    clearError();
    try {
      const exportedPackage: ExportPackage = await exportPackageCommand(outputPath);
      setExportPackage(exportedPackage);
      setStatus("ready");
      notify({ title: "Knowledge package exported", detail: exportedPackage.path, tone: "success" });
    } catch (commandError) {
      setStatus("error");
      setError(getCommandErrorMessage(commandError));
      notify({ title: "Failed to export package", detail: getCommandErrorMessage(commandError), tone: "error" });
    }
  }

  function copyText(label: string, text: string): void {
    void navigator.clipboard?.writeText(text).then(
      () => notify({ title: `${label} copied`, tone: "success" }),
      () => notify({ title: `${label} copied to clipboard unavailable`, tone: "warning" })
    );
  }

  function onAsk(): void {
    if (chatQuestion.trim().length === 0) {
      notify({ title: "Ask something first", tone: "warning" });
      return;
    }
    const question = chatQuestion.trim();
    setChatQuestion("");
    setChatTurns((current) => [
      ...current,
      {
        id: `turn-${Date.now()}`,
        question,
        response: {
          model: "local-demo",
          answer: "AI Chat is intentionally excluded from the MVP. Use Documentation, Diagrams, and Exports for generated knowledge artifacts.",
          citations: [],
        },
      },
    ]);
    notify({ title: "Chat disabled in MVP", tone: "warning" });
  }

  const topFiles = useMemo<RepositoryFile[]>(
    () => (explorerFiles.length > 0 ? explorerFiles : (scan?.repositoryFiles ?? [])),
    [explorerFiles, scan?.repositoryFiles]
  );

  const repositories = useMemo<RepositorySummary[]>(
    () => (repository ? [repository] : []),
    [repository]
  );

  const packageInsight = useMemo(
    () => ({
      documentCount: documents.length,
      diagramCount: diagrams.length,
      diagramRelationshipCount: diagrams.length > 0 ? diagrams.length * 2 : 0,
      generatedDocumentPaths: documents.map((document) => document.path),
    }),
    [documents, diagrams]
  );

  const scanSteps = useMemo(() => {
    const hasRepo = repository !== undefined && repository !== null;
    const hasFiles = (scan?.files ?? 0) > 0;
    const hasGraph = (graph?.nodeCount ?? 0) > 0 || graphNodes.length > 0;
    const defs = [
      { key: "open", done: hasRepo },
      { key: "detect", done: hasFiles },
      { key: "analyze", done: hasGraph },
    ];
    const firstUndone = defs.findIndex((def) => !def.done);
    const labels: Record<string, { label: string; description: string }> = {
      open: { label: "Open Repository", description: "Choose a folder and open it with the engine." },
      detect: { label: "Detect Technologies", description: "List files and detect the tech stack." },
      analyze: { label: "Analyze", description: "Scan files and build the knowledge graph." },
    };
    return defs.map((def, index) => {
      let status: "done" | "current" | "working" | "locked";
      if (def.done) status = "done";
      else if (processingAction === def.key) status = "working";
      else if (index === firstUndone) status = "current";
      else status = "locked";
      return { key: def.key, ...labels[def.key], status };
    });
  }, [repositoryPath, repository, scan, graph, graphNodes, processingAction]);

  const selectedFolders = useMemo(
    () => analysisScopePaths.filter((path) => !explorerFiles.some((file) => file.path === path)),
    [analysisScopePaths, explorerFiles]
  );
  const selectedFiles = useMemo(
    () => analysisScopePaths.filter((path) => explorerFiles.some((file) => file.path === path)),
    [analysisScopePaths, explorerFiles]
  );

  function openAnalyzeConfirm(): void {
    if (!repository) {
      notify({ title: "Repository required", detail: "Open a repository before analysis.", tone: "warning" });
      return;
    }
    setAnalyzeConfirmOpen(true);
  }

  function confirmAnalyze(): void {
    setAnalyzeConfirmOpen(false);
    void analyzeRepository();
  }

  function cancelAnalyze(): void {
    setAnalyzeConfirmOpen(false);
  }



  return {
    chatQuestion,
    chatTurns,
    diagrams,
    documents,
    error,
    explorerFiles,
    exportedPackage,
    graph,
    graphEdges,
    graphNodes,
    modalContent,
    nativeRuntimeAvailable,
    outputPath,
    packageInsight,
    page,
    processingAction,
    scanSteps,
    repository,
    repositoryPath,
    repositories,
    scan,
    status,
    technologies: scan?.technologies ?? [],
    scopeMode,
    setScopeMode,
    analyzeConfirmOpen,
    openAnalyzeConfirm,
    confirmAnalyze,
    cancelAnalyze,
    selectedFolders,
    selectedFiles,
    getEffectiveScopePaths,
    isScopeActive,
    isScopeDisabled,
    toggleScopeSelection,
    toasts,
    topFiles,
    uiPreferences,
    analyzeRepository,
    analysisScopeFiles: getAnalysisScopeFiles(),
    analysisScopePaths,
    clearScope,
    copyText,
    detectTechnologies,
    dismissToast: (toastId: string) => setToasts((current) => current.filter((t) => t.id !== toastId)),
    exportKnowledgePackage,
    generateDiagrams,
    generateDocuments,
    notify,
    onAnalyze: analyzeRepository,
    onAsk,
    onCloseModal: () => setModalContent(undefined),
    onCopy: copyText,
    onCopyPreview: () => {},
    onExport: exportKnowledgePackage,
    onGenerateDiagrams: generateDiagrams,
    onGenerateDocs: generateDocuments,
    onOpenRepository: () => void openRepositoryPath(),
    onPreview: setModalContent,
    onRefreshFiles: refreshFiles,
    onSelectOutput: selectOutputFolder,
    onSelectRepository: selectAndOpenRepository,
    openRepositoryDialog: selectAndOpenRepository,
    selectAndOpenRepository,
    openRepositoryPath,
    queryClient,
    refreshFiles,
    selectAllScope,
    selectOutputFolder,
    setChatQuestion,
    setOutputPath,
    setPage,
    setRepositoryPath,
    updateUiPreferences,
  };
}

export type AppController = ReturnType<typeof useAppController>;

function countFolders(files: RepositoryFile[]): number {
  const folders = new Set<string>();
  for (const file of files) {
    const parts = file.path.split(/[\\/]/).filter(Boolean);
    parts.pop();
    if (parts.length > 0) folders.add(parts.join("/"));
  }
  return folders.size;
}

function getScopePath(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  parts.pop();
  return parts.join("/");
}

function getRoutePage(): Page {
  if (typeof window === "undefined") return "dashboard";
  const hashPage = window.location.hash.replace("#/", "").replace("#", "");
  return hashPage === "dashboard" || hashPage === "explorer" || hashPage === "scanner" || hashPage === "graphs" || hashPage === "documentation" || hashPage === "diagrams" || hashPage === "exports" || hashPage === "settings" ? hashPage : "dashboard";
}

function loadUiPreferences(): UiPreferences {
  if (typeof window === "undefined" || !window.localStorage) return defaultUiPreferences;
  try {
    const stored = window.localStorage.getItem("devatlas-ui-preferences");
    return stored ? JSON.parse(stored) : defaultUiPreferences;
  } catch {
    return defaultUiPreferences;
  }
}
