export interface CommandError {
  code: string;
  message: string;
}

export const TAURI_COMMANDS = [
  "list_repositories",
  "open_repository",
  "scan_repository",
  "list_repository_files",
  "detect_technologies",
  "build_graph",
  "get_graph",
  "save_graph_snapshot",
  "list_graph_snapshots",
  "load_graph_snapshot",
  "generate_docs",
  "generate_diagrams",
  "export_package",
  "build_ai_context",
  "build_ai_embeddings",
  "build_ai_vector_store",
  "save_ai_vector_snapshot",
  "list_ai_vector_snapshots",
  "load_ai_vector_snapshot",
  "search_ai_context",
  "search_ai_context_snapshot",
  "build_ai_context_bundle",
  "build_ai_context_bundle_snapshot",
  "ask_ai",
  "ask_ai_snapshot",
  "start_chat_session",
  "list_chat_sessions",
  "list_chat_messages",
  "ask_ai_in_session",
  "ask_ai_snapshot_in_session",
  "build_repository_memory",
  "get_cloud_status",
  "get_mcp_status",
  "get_plugin_status",
  "get_performance_status",
  "get_security_status",
  "get_git_status",
  "get_backend_status",
  "get_storage_status",
] as const;

export type TauriCommandName = (typeof TAURI_COMMANDS)[number];

export interface DomainEvent {
  eventId: string;
  correlationId: string;
  eventType: DomainEventType;
  version: string;
  timestampMs: number;
  payload: DomainEventPayload;
}

export const DOMAIN_EVENT_TYPES = [
  "Repository.RepositoryOpened",
  "Scanner.ScanStarted",
  "Scanner.ScanCompleted",
  "Graph.GraphBuilt",
  "Graph.SnapshotSaved",
  "Documentation.DocumentationGenerated",
  "Diagram.DiagramGenerated",
  "Export.ExportCompleted",
  "AI.ContextBuilt",
  "AI.EmbeddingsBuilt",
  "AI.VectorStoreBuilt",
  "AI.VectorSnapshotSaved",
  "AI.RetrievalCompleted",
  "AI.PersistedRetrievalCompleted",
  "AI.ContextBundleBuilt",
  "AI.PersistedContextBundleBuilt",
  "AI.ChatResponded",
  "AI.PersistedChatResponded",
  "AI.ChatSessionStarted",
  "AI.ChatMessageSaved",
  "Repository.MemoryBuilt",
] as const;

export type DomainEventType = (typeof DOMAIN_EVENT_TYPES)[number];

export type DomainEventPayload =
  | {
      type: "RepositoryOpened";
      data: { repositoryId: string; path: string };
    }
  | {
      type: "ScanStarted";
      data: { scanId: string; repositoryId: string };
    }
  | {
      type: "ScanCompleted";
      data: { scanId: string; repositoryId: string; durationMs: number };
    }
  | {
      type: "GraphBuilt";
      data: { repositoryId: string; nodeCount: number; edgeCount: number };
    }
  | {
      type: "GraphSnapshotSaved";
      data: { repositoryId: string; snapshotId: string; nodeCount: number; edgeCount: number };
    }
  | {
      type: "DocumentationGenerated";
      data: { repositoryId: string; documentCount: number };
    }
  | {
      type: "DiagramGenerated";
      data: { repositoryId: string; diagramCount: number };
    }
  | {
      type: "ExportCompleted";
      data: { repositoryId: string; path: string };
    }
  | {
      type: "AiContextBuilt";
      data: { repositoryId: string; chunkCount: number; skippedFileCount: number };
    }
  | {
      type: "AiEmbeddingsBuilt";
      data: { repositoryId: string; embeddingCount: number; dimensions: number; model: string };
    }
  | {
      type: "AiVectorStoreBuilt";
      data: { repositoryId: string; embeddingCount: number; dimensions: number; model: string };
    }
  | {
      type: "AiVectorSnapshotSaved";
      data: {
        repositoryId: string;
        snapshotId: string;
        embeddingCount: number;
        dimensions: number;
        model: string;
      };
    }
  | {
      type: "AiRetrievalCompleted";
      data: { repositoryId: string; query: string; matchCount: number };
    }
  | {
      type: "AiPersistedRetrievalCompleted";
      data: { repositoryId: string; snapshotId: string; query: string; matchCount: number };
    }
  | {
      type: "AiContextBundleBuilt";
      data: {
        repositoryId: string;
        query: string;
        sourceCount: number;
        tokenEstimate: number;
        truncated: boolean;
      };
    }
  | {
      type: "AiPersistedContextBundleBuilt";
      data: {
        repositoryId: string;
        snapshotId: string;
        query: string;
        sourceCount: number;
        tokenEstimate: number;
        truncated: boolean;
      };
    }
  | {
      type: "AiChatResponded";
      data: { repositoryId: string; question: string; citationCount: number; model: string };
    }
  | {
      type: "AiPersistedChatResponded";
      data: {
        repositoryId: string;
        snapshotId: string;
        question: string;
        citationCount: number;
        model: string;
      };
    }
  | {
      type: "AiChatSessionStarted";
      data: { repositoryId: string; sessionId: string; title: string };
    }
  | {
      type: "AiChatMessageSaved";
      data: { repositoryId: string; sessionId: string; messageId: string; role: string };
    }
  | {
      type: "RepositoryMemoryBuilt";
      data: {
        repositoryId: string;
        filesCount: number;
        graphNodes: number;
        aiContextChunks: number;
      };
    };

export interface RepositorySummary {
  repositoryId: string;
  name: string;
  path: string;
}

export interface Technology {
  category: string;
  name: string;
  version?: string | null;
}

export interface RepositoryFile {
  path: string;
  extension?: string | null;
  sizeBytes: number;
}

export interface ScanResult {
  scanId: string;
  repositoryId: string;
  files: number;
  folders: number;
  durationMs: number;
  technologies: Technology[];
  repositoryFiles: RepositoryFile[];
}

export interface GraphSummary {
  nodeCount: number;
  edgeCount: number;
}

export interface GraphNode {
  id: string;
  nodeType: string;
  name: string;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  edgeType: string;
}

export interface KnowledgeGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface GraphSnapshotSummary {
  id: string;
  repositoryId: string;
  scanId?: string | null;
  nodeCount: number;
  edgeCount: number;
  createdAtMs: number;
}

export interface GraphSnapshot {
  id: string;
  repositoryId: string;
  scanId?: string | null;
  graph: KnowledgeGraph;
  nodeCount: number;
  edgeCount: number;
  createdAtMs: number;
}

export interface GeneratedDocument {
  id: string;
  path: string;
  documentType: string;
  content: string;
  semanticPlan: DocumentationPlan;
  quality: DocumentationQuality;
}

export interface DocumentationPlan {
  audience: string;
  intent: string;
  sections: DocumentationSectionPlan[];
  evidenceSources: string[];
}

export interface DocumentationSectionPlan {
  title: string;
  purpose: string;
  evidenceType: string;
  requiredSignals: string[];
}

export interface DocumentationQuality {
  coverageScore: number;
  semanticScore: number;
  sourceCount: number;
  symbolCount: number;
  graphEdgeCount: number;
  warnings: string[];
}

export interface DiagramResult {
  id: string;
  path: string;
  diagramType: string;
  format: string;
  content: string;
}

export interface ExportPackage {
  id: string;
  path: string;
  artifactsDir: string;
  artifactCount: number;
}

export interface AiContextChunk {
  id: string;
  path: string;
  chunkIndex: number;
  startLine: number;
  endLine: number;
  content: string;
  charCount: number;
  tokenEstimate: number;
}

export interface SkippedContextFile {
  path: string;
  reason: string;
}

export interface AiContextResult {
  chunks: AiContextChunk[];
  skippedFiles: SkippedContextFile[];
}

export interface AiEmbeddingVector {
  id: string;
  chunkId: string;
  path: string;
  dimensions: number;
  model: string;
  values: number[];
}

export interface AiEmbeddingResult {
  embeddings: AiEmbeddingVector[];
  dimensions: number;
  model: string;
}

export interface AiVectorStoreResult {
  embeddingCount: number;
  dimensions: number;
  model: string;
}

export interface AiVectorSnapshotSummary {
  id: string;
  repositoryId: string;
  scanId?: string | null;
  embeddingCount: number;
  dimensions: number;
  model: string;
  createdAtMs: number;
}

export interface AiVectorSnapshot {
  id: string;
  repositoryId: string;
  scanId?: string | null;
  embeddings: AiEmbeddingVector[];
  embeddingCount: number;
  dimensions: number;
  model: string;
  createdAtMs: number;
}

export interface AiRetrievalMatch {
  chunkId: string;
  path: string;
  startLine: number;
  endLine: number;
  content: string;
  score: number;
  vectorScore: number;
  lexicalScore: number;
}

export interface AiRetrievalResult {
  query: string;
  matches: AiRetrievalMatch[];
}

export interface AiContextBundleSource {
  sourceId: string;
  chunkId: string;
  path: string;
  startLine: number;
  endLine: number;
  score: number;
  tokenEstimate: number;
}

export interface AiContextBundle {
  query: string;
  content: string;
  sources: AiContextBundleSource[];
  tokenEstimate: number;
  maxTokens: number;
  truncated: boolean;
}

export interface AiChatCitation {
  sourceId: string;
  path: string;
  startLine: number;
  endLine: number;
  score: number;
}

export interface AiChatResponse {
  question: string;
  answer: string;
  citations: AiChatCitation[];
  context: AiContextBundle;
  model: string;
}

export type ChatMessageRole = "User" | "Assistant" | "System";

export interface ChatSession {
  id: string;
  repositoryId: string;
  title: string;
  createdAtMs: number;
  updatedAtMs: number;
}

export interface ChatMessage {
  id: string;
  repositoryId: string;
  sessionId: string;
  role: ChatMessageRole;
  content: string;
  model?: string | null;
  citationCount: number;
  createdAtMs: number;
}

export interface RepositoryMemoryTechnology {
  category: string;
  name: string;
  version?: string | null;
}

export interface RepositoryMemory {
  repositoryId: string;
  repositoryName: string;
  path: string;
  scanId?: string | null;
  filesCount: number;
  foldersCount: number;
  technologies: RepositoryMemoryTechnology[];
  graphNodes: number;
  graphEdges: number;
  documentCount: number;
  diagramCount: number;
  lastExportPath?: string | null;
  aiContextChunks: number;
  aiEmbeddingCount: number;
  aiModel?: string | null;
  recentQuestions: string[];
  updatedAtMs: number;
}

export interface CloudReadiness {
  enabled: boolean;
  status: string;
  supportLevel: string;
  tenantTypes: string[];
  syncModes: string[];
  deploymentModels: string[];
  networkRequired: boolean;
  cloudTablesEnabled: boolean;
}

export interface McpReadiness {
  enabled: boolean;
  status: string;
  supportLevel: string;
  transports: string[];
  resources: string[];
  tools: string[];
  prompts: string[];
  authenticationRequired: boolean;
  runtimeCrateEnabled: boolean;
}

export interface PluginReadiness {
  enabled: boolean;
  status: string;
  supportLevel: string;
  categories: string[];
  lifecycle: string[];
  permissions: string[];
  manifestRequiredFields: string[];
  sandboxRequired: boolean;
  marketplaceEnabled: boolean;
  runtimeCrateEnabled: boolean;
  registryTableEnabled: boolean;
  networkAccessEnabled: boolean;
}

export interface PerformanceReadiness {
  enabled: boolean;
  status: string;
  supportLevel: string;
  analysisDomains: string[];
  metrics: string[];
  riskLevels: string[];
  thresholds: string[];
  reportTypes: string[];
  runtimeCrateEnabled: boolean;
  reportTablesEnabled: boolean;
  profilingEnabled: boolean;
  recommendationsEnabled: boolean;
}

export interface SecurityReadiness {
  enabled: boolean;
  status: string;
  supportLevel: string;
  securityDomains: string[];
  secretCategories: string[];
  dependencySources: string[];
  owaspMappings: string[];
  riskLevels: string[];
  reportTypes: string[];
  runtimeCrateEnabled: boolean;
  reportTablesEnabled: boolean;
  secretValueExportEnabled: boolean;
  vulnerabilityNetworkLookupEnabled: boolean;
  dashboardEnabled: boolean;
}

export interface GitReadiness {
  enabled: boolean;
  status: string;
  supportLevel: string;
  dataSources: string[];
  analysisDomains: string[];
  graphNodeTypes: string[];
  relationshipTypes: string[];
  reportTypes: string[];
  runtimeCrateEnabled: boolean;
  historyMutationEnabled: boolean;
  repositoryTimelineEnabled: boolean;
  ownershipAnalysisEnabled: boolean;
  hotspotAnalysisEnabled: boolean;
  driftDetectionEnabled: boolean;
}

export interface BackendReadiness {
  architecture: string;
  commandLayer: string;
  structuredErrors: boolean;
  eventBusEnabled: boolean;
  storageEnabled: boolean;
  businessLogicInRust: boolean;
  networkCallsEnabled: boolean;
  serviceCrates: string[];
  futureCrates: string[];
}

export interface StorageReadiness {
  primaryStore: string;
  operationalTables: string[];
  snapshotTables: string[];
  futureTables: string[];
  vectorStoreEnabled: boolean;
  vectorStore: string;
  searchIndexEnabled: boolean;
  searchIndex: string;
  graphEngineEnabled: boolean;
  graphPersistence: string;
  jsonSnapshotsEnabled: boolean;
  cloudTablesEnabled: boolean;
}
