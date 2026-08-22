import { create } from "zustand";
import type {
  DiagramResult,
  ExportPackage,
  GeneratedDocument,
  GraphSummary,
  GraphNode,
  GraphEdge,
  RepositorySummary,
  ScanResult,
} from "../types/contracts";

export type AppStatus = "idle" | "working" | "ready" | "error";

export interface AppStoreState {
  repository?: RepositorySummary;
  scan?: ScanResult;
  graph?: GraphSummary;
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
  documents: GeneratedDocument[];
  diagrams: DiagramResult[];
  exportPackage?: ExportPackage;
  status: AppStatus;
  error?: string;
  setRepository: (repository: RepositorySummary) => void;
  setScan: (scan?: ScanResult) => void;
  setGraph: (graph?: GraphSummary) => void;
  setGraphNodes: (graphNodes: GraphNode[]) => void;
  setGraphEdges: (graphEdges: GraphEdge[]) => void;
  setDocuments: (documents: GeneratedDocument[]) => void;
  setDiagrams: (diagrams: DiagramResult[]) => void;
  setExportPackage: (exportPackage?: ExportPackage) => void;
  setStatus: (status: AppStatus) => void;
  setError: (error: string | undefined) => void;
}

export const useAppStore = create<AppStoreState>((set) => ({
  graphNodes: [],
  graphEdges: [],
  documents: [],
  diagrams: [],
  status: "idle",
  setRepository: (repository) => set({ repository, error: undefined }),
  setScan: (scan) => set({ scan, error: undefined }),
  setGraph: (graph) => set({ graph, error: undefined }),
  setGraphNodes: (graphNodes: GraphNode[]) => set({ graphNodes, error: undefined }),
  setGraphEdges: (graphEdges: GraphEdge[]) => set({ graphEdges, error: undefined }),
  setDocuments: (documents) => set({ documents, error: undefined }),
  setDiagrams: (diagrams) => set({ diagrams, error: undefined }),
  setExportPackage: (exportPackage) => set({ exportPackage, error: undefined }),
  setStatus: (status) => set({ status }),
  setError: (error) => set({ error, status: error ? "error" : "idle" }),
}));
