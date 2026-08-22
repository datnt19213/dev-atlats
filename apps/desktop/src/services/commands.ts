import { invoke } from "@tauri-apps/api/core";
import type {
  AiChatResponse,
  AiContextBundle,
  AiContextResult,
  AiEmbeddingResult,
  AiRetrievalResult,
  AiVectorSnapshot,
  AiVectorSnapshotSummary,
  AiVectorStoreResult,
  BackendReadiness,
  ChatMessage,
  ChatSession,
  CloudReadiness,
  CommandError,
  DiagramResult,
  ExportPackage,
  GeneratedDocument,
  GitReadiness,
  GraphSnapshot,
  GraphSnapshotSummary,
  GraphSummary,
  KnowledgeGraph,
  McpReadiness,
  PerformanceReadiness,
  PluginReadiness,
  RepositoryMemory,
  RepositoryFile,
  RepositorySummary,
  ScanResult,
  SecurityReadiness,
  StorageReadiness,
  TauriCommandName,
  Technology,
} from "../types/contracts";

export function getCommandErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (isCommandError(error)) {
    return error.message;
  }
  return String(error);
}

function isCommandError(error: unknown): error is CommandError {
  if (typeof error !== "object" || error === null) {
    return false;
  }
  const candidate = error as Record<string, unknown>;
  return typeof candidate.code === "string" && typeof candidate.message === "string";
}

function invokeCommand<TResponse>(
  command: TauriCommandName,
  args?: Record<string, unknown>,
): Promise<TResponse> {
  if (!isTauriRuntime()) {
    return Promise.reject(
      new Error(
        "DevAtlas native commands require the Tauri desktop runtime. Run `yarn dev` and use the opened desktop window instead of a browser-only localhost tab.",
      ),
    );
  }
  return invoke<TResponse>(command, args);
}

export function isTauriRuntime(): boolean {
  const runtime = globalThis as typeof globalThis & {
    __TAURI_INTERNALS__?: unknown;
  };
  return typeof runtime.__TAURI_INTERNALS__ !== "undefined";
}

export async function openRepository(path: string): Promise<RepositorySummary> {
  return invokeCommand<RepositorySummary>("open_repository", { path });
}

export async function listRepositories(): Promise<RepositorySummary[]> {
  return invokeCommand<RepositorySummary[]>("list_repositories");
}

export async function scanRepository(maxFiles?: number, includePaths?: string[]): Promise<ScanResult> {
  return invokeCommand<ScanResult>("scan_repository", { maxFiles, includePaths });
}

export async function listRepositoryFiles(): Promise<RepositoryFile[]> {
  return invokeCommand<RepositoryFile[]>("list_repository_files");
}

export async function detectTechnologies(): Promise<Technology[]> {
  return invokeCommand<Technology[]>("detect_technologies");
}

export async function buildGraph(): Promise<GraphSummary> {
  return invokeCommand<GraphSummary>("build_graph");
}

export async function getGraph(): Promise<KnowledgeGraph> {
  return invokeCommand<KnowledgeGraph>("get_graph");
}

export async function saveGraphSnapshot(): Promise<GraphSnapshotSummary> {
  return invokeCommand<GraphSnapshotSummary>("save_graph_snapshot");
}

export async function listGraphSnapshots(): Promise<GraphSnapshotSummary[]> {
  return invokeCommand<GraphSnapshotSummary[]>("list_graph_snapshots");
}

export async function loadGraphSnapshot(snapshotId: string): Promise<GraphSnapshot> {
  return invokeCommand<GraphSnapshot>("load_graph_snapshot", { snapshotId });
}

export async function generateDocs(): Promise<GeneratedDocument[]> {
  return invokeCommand<GeneratedDocument[]>("generate_docs");
}

export async function generateDiagrams(): Promise<DiagramResult[]> {
  return invokeCommand<DiagramResult[]>("generate_diagrams");
}

export async function exportPackage(outputDir: string): Promise<ExportPackage> {
  return invokeCommand<ExportPackage>("export_package", { outputDir });
}

export async function buildAiContext(): Promise<AiContextResult> {
  return invokeCommand<AiContextResult>("build_ai_context");
}

export async function buildAiEmbeddings(): Promise<AiEmbeddingResult> {
  return invokeCommand<AiEmbeddingResult>("build_ai_embeddings");
}

export async function buildAiVectorStore(): Promise<AiVectorStoreResult> {
  return invokeCommand<AiVectorStoreResult>("build_ai_vector_store");
}

export async function saveAiVectorSnapshot(): Promise<AiVectorSnapshotSummary> {
  return invokeCommand<AiVectorSnapshotSummary>("save_ai_vector_snapshot");
}

export async function listAiVectorSnapshots(): Promise<AiVectorSnapshotSummary[]> {
  return invokeCommand<AiVectorSnapshotSummary[]>("list_ai_vector_snapshots");
}

export async function loadAiVectorSnapshot(snapshotId: string): Promise<AiVectorSnapshot> {
  return invokeCommand<AiVectorSnapshot>("load_ai_vector_snapshot", { snapshotId });
}

export async function searchAiContext(
  query: string,
  limit?: number,
): Promise<AiRetrievalResult> {
  return invokeCommand<AiRetrievalResult>("search_ai_context", { query, limit });
}

export async function searchAiContextSnapshot(
  snapshotId: string,
  query: string,
  limit?: number,
): Promise<AiRetrievalResult> {
  return invokeCommand<AiRetrievalResult>("search_ai_context_snapshot", { snapshotId, query, limit });
}

export async function buildAiContextBundle(
  query: string,
  limit?: number,
  maxTokens?: number,
): Promise<AiContextBundle> {
  return invokeCommand<AiContextBundle>("build_ai_context_bundle", { query, limit, maxTokens });
}

export async function buildAiContextBundleSnapshot(
  snapshotId: string,
  query: string,
  limit?: number,
  maxTokens?: number,
): Promise<AiContextBundle> {
  return invokeCommand<AiContextBundle>("build_ai_context_bundle_snapshot", {
    snapshotId,
    query,
    limit,
    maxTokens,
  });
}

export async function askAi(
  question: string,
  limit?: number,
  maxContextTokens?: number,
): Promise<AiChatResponse> {
  return invokeCommand<AiChatResponse>("ask_ai", { question, limit, maxContextTokens });
}

export async function askAiSnapshot(
  snapshotId: string,
  question: string,
  limit?: number,
  maxContextTokens?: number,
): Promise<AiChatResponse> {
  return invokeCommand<AiChatResponse>("ask_ai_snapshot", {
    snapshotId,
    question,
    limit,
    maxContextTokens,
  });
}

export async function startChatSession(title?: string): Promise<ChatSession> {
  return invokeCommand<ChatSession>("start_chat_session", { title });
}

export async function listChatSessions(): Promise<ChatSession[]> {
  return invokeCommand<ChatSession[]>("list_chat_sessions");
}

export async function listChatMessages(sessionId: string): Promise<ChatMessage[]> {
  return invokeCommand<ChatMessage[]>("list_chat_messages", { sessionId });
}

export async function askAiInSession(
  sessionId: string,
  question: string,
  limit?: number,
  maxContextTokens?: number,
): Promise<AiChatResponse> {
  return invokeCommand<AiChatResponse>("ask_ai_in_session", {
    sessionId,
    question,
    limit,
    maxContextTokens,
  });
}

export async function askAiSnapshotInSession(
  sessionId: string,
  snapshotId: string,
  question: string,
  limit?: number,
  maxContextTokens?: number,
): Promise<AiChatResponse> {
  return invokeCommand<AiChatResponse>("ask_ai_snapshot_in_session", {
    sessionId,
    snapshotId,
    question,
    limit,
    maxContextTokens,
  });
}

export async function buildRepositoryMemory(): Promise<RepositoryMemory> {
  return invokeCommand<RepositoryMemory>("build_repository_memory");
}

export async function getCloudStatus(): Promise<CloudReadiness> {
  return invokeCommand<CloudReadiness>("get_cloud_status");
}

export async function getMcpStatus(): Promise<McpReadiness> {
  return invokeCommand<McpReadiness>("get_mcp_status");
}

export async function getPluginStatus(): Promise<PluginReadiness> {
  return invokeCommand<PluginReadiness>("get_plugin_status");
}

export async function getPerformanceStatus(): Promise<PerformanceReadiness> {
  return invokeCommand<PerformanceReadiness>("get_performance_status");
}

export async function getSecurityStatus(): Promise<SecurityReadiness> {
  return invokeCommand<SecurityReadiness>("get_security_status");
}

export async function getGitStatus(): Promise<GitReadiness> {
  return invokeCommand<GitReadiness>("get_git_status");
}

export async function getBackendStatus(): Promise<BackendReadiness> {
  return invokeCommand<BackendReadiness>("get_backend_status");
}

export async function getStorageStatus(): Promise<StorageReadiness> {
  return invokeCommand<StorageReadiness>("get_storage_status");
}
