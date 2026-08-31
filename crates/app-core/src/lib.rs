use devatlas_ai_engine::{
    AiChatService, AiContextBuilderService, AiContextService, AiEmbeddingService,
    AiRetrievalService, AiVectorStoreService, ChatRequest, ChatResponse, ContextBuildResult,
    ContextBundle, ContextBundleRequest, EmbeddingBuildResult, EmbeddingVector, RetrievalQuery,
    RetrievalResult, VectorStoreBuildResult,
};
use devatlas_common::{
    now_ms, stable_id, AiVectorEmbedding, AiVectorSnapshot, AiVectorSnapshotId, BackendReadiness,
    ChatMessage, ChatMessageId, ChatMessageRole, ChatSession, ChatSessionId, CloudReadiness,
    DevAtlasResult, DiagramResult, DomainEvent, ExportPackage, GeneratedDocument, GitReadiness,
    GraphSnapshot, GraphSnapshotId, KnowledgeGraph, McpReadiness, PerformanceReadiness,
    PluginReadiness, Repository, RepositoryId, RepositoryMemory, RepositoryMemoryTechnology,
    RepositoryPath, ScanResult, SecurityReadiness, StorageReadiness,
};
use devatlas_docs_engine::DocumentationService;
use devatlas_export_engine::ExportService;
use devatlas_graph_engine::GraphService;
use devatlas_parser_engine::ParserService;
use devatlas_scanner_engine::{ScanOptions, ScannerService};
pub use devatlas_storage_engine::SqliteStorage;
use devatlas_storage_engine::{StoragePlan, StorageService};
use devatlas_uml_engine::UmlService;
use std::path::Path;
use tokio::sync::broadcast;

pub struct AppService;

pub struct RepositoryMemoryInput<'a> {
    pub repository: &'a Repository,
    pub scan: Option<&'a ScanResult>,
    pub graph: Option<&'a KnowledgeGraph>,
    pub documents: &'a [GeneratedDocument],
    pub diagrams: &'a [DiagramResult],
    pub export_package: Option<&'a ExportPackage>,
    pub ai_context: Option<&'a ContextBuildResult>,
    pub ai_embeddings: Option<&'a EmbeddingBuildResult>,
    pub recent_questions: &'a [String],
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: DomainEvent) -> usize {
        self.sender.send(event).unwrap_or(0)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

impl AppService {
    pub fn open_repository(path: impl Into<String>) -> DevAtlasResult<Repository> {
        let path_string = path.into();
        let repository_path = RepositoryPath::new(&path_string)?;
        let name = repository_path
            .as_path()
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "repository".to_string());
        Ok(Repository {
            id: RepositoryId(stable_id("repo", &path_string)),
            name,
            path: repository_path,
            created_at_ms: now_ms(),
        })
    }

    pub fn scan_repository(repository: &Repository) -> DevAtlasResult<ScanResult> {
        ScannerService::scan_repository(repository.id.clone(), &repository.path)
    }

    pub fn scan_repository_with_options(
        repository: &Repository,
        options: &ScanOptions,
    ) -> DevAtlasResult<ScanResult> {
        ScannerService::scan_repository_with_options(
            repository.id.clone(),
            &repository.path,
            options,
        )
    }

    pub fn build_graph(scan: &ScanResult) -> KnowledgeGraph {
        GraphService::build_graph(scan)
    }

    pub fn build_graph_for_repository(
        repository: &Repository,
        scan: &ScanResult,
    ) -> DevAtlasResult<KnowledgeGraph> {
        let parsed = ParserService::parse_repository(&repository.path, &scan.files)?;
        Ok(GraphService::build_graph_with_parsed(scan, &parsed))
    }

    pub fn generate_docs(scan: &ScanResult, graph: &KnowledgeGraph) -> Vec<GeneratedDocument> {
        DocumentationService::generate_documents(scan, graph)
    }

    pub fn generate_diagrams(scan: &ScanResult, graph: &KnowledgeGraph) -> Vec<DiagramResult> {
        UmlService::generate_diagrams(scan, graph)
    }

    pub fn create_graph_snapshot(
        repository: &Repository,
        scan: Option<&ScanResult>,
        graph: &KnowledgeGraph,
    ) -> GraphSnapshot {
        let created_at_ms = now_ms();
        let scan_id = scan.map(|scan_result| scan_result.scan_id.clone());
        let snapshot_key = format!(
            "{}-{}-{}-{}-{}",
            repository.id.0,
            scan_id
                .as_ref()
                .map(|value| value.0.as_str())
                .unwrap_or("no-scan"),
            graph.nodes.len(),
            graph.edges.len(),
            created_at_ms
        );
        GraphSnapshot {
            id: GraphSnapshotId(stable_id("graph-snapshot", &snapshot_key)),
            repository_id: repository.id.clone(),
            scan_id,
            graph: graph.clone(),
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            created_at_ms,
        }
    }

    pub fn build_ai_context(
        repository: &Repository,
        scan: &ScanResult,
    ) -> DevAtlasResult<ContextBuildResult> {
        AiContextService::build_context(&repository.path, &scan.files)
    }

    pub fn build_ai_embeddings(
        repository: &Repository,
        scan: &ScanResult,
    ) -> DevAtlasResult<EmbeddingBuildResult> {
        let context = Self::build_ai_context(repository, scan)?;
        AiEmbeddingService::build_embeddings(&context)
    }

    pub fn build_ai_vector_store(
        repository: &Repository,
        scan: &ScanResult,
    ) -> DevAtlasResult<VectorStoreBuildResult> {
        let embeddings = Self::build_ai_embeddings(repository, scan)?;
        AiVectorStoreService::build_store(&embeddings)
    }

    pub fn create_ai_vector_snapshot(
        repository: &Repository,
        scan: Option<&ScanResult>,
        vector_store: &VectorStoreBuildResult,
    ) -> AiVectorSnapshot {
        let created_at_ms = now_ms();
        let scan_id = scan.map(|scan_result| scan_result.scan_id.clone());
        let snapshot_key = format!(
            "{}-{}-{}-{}-{}",
            repository.id.0,
            scan_id
                .as_ref()
                .map(|value| value.0.as_str())
                .unwrap_or("no-scan"),
            vector_store.embedding_count,
            vector_store.model,
            created_at_ms
        );
        AiVectorSnapshot {
            id: AiVectorSnapshotId(stable_id("ai-vector-snapshot", &snapshot_key)),
            repository_id: repository.id.clone(),
            scan_id,
            embeddings: vector_store
                .embeddings
                .iter()
                .map(|embedding| AiVectorEmbedding {
                    id: embedding.id.clone(),
                    chunk_id: embedding.chunk_id.clone(),
                    path: embedding.path.clone(),
                    dimensions: embedding.dimensions,
                    model: embedding.model.clone(),
                    values: embedding.values.clone(),
                })
                .collect(),
            embedding_count: vector_store.embedding_count,
            dimensions: vector_store.dimensions,
            model: vector_store.model.clone(),
            created_at_ms,
        }
    }

    pub fn search_ai_context(
        repository: &Repository,
        scan: &ScanResult,
        query: impl Into<String>,
        limit: Option<usize>,
    ) -> DevAtlasResult<RetrievalResult> {
        let context = Self::build_ai_context(repository, scan)?;
        let embeddings = AiEmbeddingService::build_embeddings(&context)?;
        AiRetrievalService::search(&context, &embeddings, &RetrievalQuery::new(query, limit))
    }

    pub fn search_ai_context_snapshot(
        repository: &Repository,
        scan: &ScanResult,
        snapshot: &AiVectorSnapshot,
        query: impl Into<String>,
        limit: Option<usize>,
    ) -> DevAtlasResult<RetrievalResult> {
        let context = Self::build_ai_context(repository, scan)?;
        let embeddings = snapshot_to_embedding_result(snapshot);
        AiRetrievalService::search(&context, &embeddings, &RetrievalQuery::new(query, limit))
    }

    pub fn build_ai_context_bundle_snapshot(
        repository: &Repository,
        scan: &ScanResult,
        snapshot: &AiVectorSnapshot,
        query: impl Into<String>,
        limit: Option<usize>,
        max_tokens: Option<usize>,
    ) -> DevAtlasResult<ContextBundle> {
        let context = Self::build_ai_context(repository, scan)?;
        let embeddings = snapshot_to_embedding_result(snapshot);
        AiContextBuilderService::build_bundle(
            &context,
            &embeddings,
            &ContextBundleRequest::new(query, limit, max_tokens),
        )
    }

    pub fn build_ai_context_bundle(
        repository: &Repository,
        scan: &ScanResult,
        query: impl Into<String>,
        limit: Option<usize>,
        max_tokens: Option<usize>,
    ) -> DevAtlasResult<ContextBundle> {
        let context = Self::build_ai_context(repository, scan)?;
        let embeddings = AiEmbeddingService::build_embeddings(&context)?;
        AiContextBuilderService::build_bundle(
            &context,
            &embeddings,
            &ContextBundleRequest::new(query, limit, max_tokens),
        )
    }

    pub fn ask_ai(
        repository: &Repository,
        scan: &ScanResult,
        question: impl Into<String>,
        limit: Option<usize>,
        max_context_tokens: Option<usize>,
    ) -> DevAtlasResult<ChatResponse> {
        let context = Self::build_ai_context(repository, scan)?;
        let embeddings = AiEmbeddingService::build_embeddings(&context)?;
        AiChatService::answer(
            &context,
            &embeddings,
            &ChatRequest::new(question, limit, max_context_tokens),
        )
    }

    pub fn ask_ai_snapshot(
        repository: &Repository,
        scan: &ScanResult,
        snapshot: &AiVectorSnapshot,
        question: impl Into<String>,
        limit: Option<usize>,
        max_context_tokens: Option<usize>,
    ) -> DevAtlasResult<ChatResponse> {
        let context = Self::build_ai_context(repository, scan)?;
        let embeddings = snapshot_to_embedding_result(snapshot);
        AiChatService::answer(
            &context,
            &embeddings,
            &ChatRequest::new(question, limit, max_context_tokens),
        )
    }

    pub fn create_chat_session(repository: &Repository, title: Option<String>) -> ChatSession {
        let created_at_ms = now_ms();
        let title = title
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "New repository chat".to_string());
        let session_key = format!("{}-{}-{}", repository.id.0, title, created_at_ms);
        ChatSession {
            id: ChatSessionId(stable_id("chat-session", &session_key)),
            repository_id: repository.id.clone(),
            title,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn create_chat_message(
        repository: &Repository,
        session: &ChatSession,
        role: ChatMessageRole,
        content: impl Into<String>,
        model: Option<String>,
        citation_count: usize,
    ) -> ChatMessage {
        let created_at_ms = now_ms();
        let content = content.into();
        let message_key = format!(
            "{}-{}-{}-{}-{}",
            repository.id.0,
            session.id.0,
            role.as_str(),
            content,
            created_at_ms
        );
        ChatMessage {
            id: ChatMessageId(stable_id("chat-message", &message_key)),
            repository_id: repository.id.clone(),
            session_id: session.id.clone(),
            role,
            content,
            model,
            citation_count,
            created_at_ms,
        }
    }

    pub fn build_repository_memory(input: RepositoryMemoryInput<'_>) -> RepositoryMemory {
        let technologies = input
            .scan
            .map(|scan_result| {
                scan_result
                    .technologies
                    .iter()
                    .map(|technology| RepositoryMemoryTechnology {
                        category: technology.category.as_str().to_string(),
                        name: technology.name.clone(),
                        version: technology.version.clone(),
                    })
                    .collect::<Vec<RepositoryMemoryTechnology>>()
            })
            .unwrap_or_default();

        RepositoryMemory {
            repository_id: input.repository.id.0.clone(),
            repository_name: input.repository.name.clone(),
            path: input
                .repository
                .path
                .as_path()
                .to_string_lossy()
                .to_string(),
            scan_id: input.scan.map(|scan_result| scan_result.scan_id.0.clone()),
            files_count: input
                .scan
                .map(|scan_result| scan_result.files_count)
                .unwrap_or(0),
            folders_count: input
                .scan
                .map(|scan_result| scan_result.folders_count)
                .unwrap_or(0),
            technologies,
            graph_nodes: input
                .graph
                .map(|knowledge_graph| knowledge_graph.nodes.len())
                .unwrap_or(0),
            graph_edges: input
                .graph
                .map(|knowledge_graph| knowledge_graph.edges.len())
                .unwrap_or(0),
            document_count: input.documents.len(),
            diagram_count: input.diagrams.len(),
            last_export_path: input.export_package.map(|package| package.path.clone()),
            ai_context_chunks: input
                .ai_context
                .map(|context| context.chunks.len())
                .unwrap_or(0),
            ai_embedding_count: input
                .ai_embeddings
                .map(|embeddings| embeddings.embeddings.len())
                .unwrap_or(0),
            ai_model: input
                .ai_embeddings
                .map(|embeddings| embeddings.model.clone()),
            recent_questions: input.recent_questions.to_vec(),
            updated_at_ms: now_ms(),
        }
    }

    pub fn cloud_readiness() -> CloudReadiness {
        CloudReadiness {
            enabled: false,
            status: "future-only".to_string(),
            support_level: "strategic-design".to_string(),
            tenant_types: vec![
                "Individual".to_string(),
                "Team".to_string(),
                "Organization".to_string(),
                "Enterprise".to_string(),
            ],
            sync_modes: vec![
                "Manual".to_string(),
                "Scheduled".to_string(),
                "Real-Time".to_string(),
            ],
            deployment_models: vec![
                "Shared SaaS".to_string(),
                "Dedicated Tenant".to_string(),
                "Self Hosted".to_string(),
                "Air-Gapped".to_string(),
            ],
            network_required: false,
            cloud_tables_enabled: false,
        }
    }

    pub fn mcp_readiness() -> McpReadiness {
        McpReadiness {
            enabled: false,
            status: "future-only".to_string(),
            support_level: "strategic-design".to_string(),
            transports: vec![
                "STDIO".to_string(),
                "HTTP".to_string(),
                "WebSocket".to_string(),
            ],
            resources: vec![
                "repository://current".to_string(),
                "architecture://overview".to_string(),
                "graph://repository".to_string(),
                "api://all".to_string(),
                "database://schema".to_string(),
                "git://history".to_string(),
                "docs://generated".to_string(),
            ],
            tools: vec![
                "scan_repository".to_string(),
                "build_graph".to_string(),
                "query_graph".to_string(),
                "find_dependencies".to_string(),
                "explain_module".to_string(),
                "generate_diagram".to_string(),
                "generate_docs".to_string(),
                "ask_repository".to_string(),
                "find_hotspots".to_string(),
                "analyze_security".to_string(),
                "analyze_performance".to_string(),
                "build_context_package".to_string(),
                "build_knowledge_package".to_string(),
            ],
            prompts: vec![
                "Architecture Review".to_string(),
                "Security Review".to_string(),
                "Onboarding".to_string(),
                "Refactoring".to_string(),
                "ADR".to_string(),
                "Repository Intelligence".to_string(),
            ],
            authentication_required: false,
            runtime_crate_enabled: false,
        }
    }

    pub fn plugin_readiness() -> PluginReadiness {
        PluginReadiness {
            enabled: false,
            status: "future-only".to_string(),
            support_level: "strategic-design".to_string(),
            categories: vec![
                "Scanner".to_string(),
                "AI".to_string(),
                "Diagram".to_string(),
                "Export".to_string(),
                "Analysis".to_string(),
                "Integration".to_string(),
                "MCP".to_string(),
            ],
            lifecycle: vec![
                "Install".to_string(),
                "Validate".to_string(),
                "Load".to_string(),
                "Initialize".to_string(),
                "Run".to_string(),
                "Unload".to_string(),
                "Remove".to_string(),
            ],
            permissions: vec![
                "Read Repository".to_string(),
                "Read Settings".to_string(),
                "Generate Reports".to_string(),
                "Access AI Providers".to_string(),
                "Access Network".to_string(),
                "Create Files".to_string(),
                "Modify Exports".to_string(),
            ],
            manifest_required_fields: vec![
                "id".to_string(),
                "name".to_string(),
                "version".to_string(),
                "author".to_string(),
                "description".to_string(),
                "engineVersion".to_string(),
                "permissions".to_string(),
            ],
            sandbox_required: true,
            marketplace_enabled: false,
            runtime_crate_enabled: false,
            registry_table_enabled: false,
            network_access_enabled: false,
        }
    }

    pub fn performance_readiness() -> PerformanceReadiness {
        PerformanceReadiness {
            enabled: false,
            status: "future-only".to_string(),
            support_level: "strategic-design".to_string(),
            analysis_domains: vec![
                "Code Complexity".to_string(),
                "Dependency Complexity".to_string(),
                "Repository Scale".to_string(),
                "Architectural Complexity".to_string(),
                "API Performance".to_string(),
                "Database Performance".to_string(),
                "Frontend Performance".to_string(),
            ],
            metrics: vec![
                "Cyclomatic Complexity".to_string(),
                "Cognitive Complexity".to_string(),
                "Dependency Density".to_string(),
                "Graph Density".to_string(),
                "Coupling".to_string(),
                "Cohesion".to_string(),
            ],
            risk_levels: vec![
                "Low".to_string(),
                "Medium".to_string(),
                "High".to_string(),
                "Critical".to_string(),
            ],
            thresholds: vec![
                "Medium Function: 50+ lines".to_string(),
                "High Function: 100+ lines".to_string(),
                "Critical Function: 200+ lines".to_string(),
            ],
            report_types: vec![
                "Complexity Report".to_string(),
                "Dependency Report".to_string(),
                "Scalability Report".to_string(),
                "Optimization Recommendations".to_string(),
            ],
            runtime_crate_enabled: false,
            report_tables_enabled: false,
            profiling_enabled: false,
            recommendations_enabled: false,
        }
    }

    pub fn security_readiness() -> SecurityReadiness {
        SecurityReadiness {
            enabled: false,
            status: "future-only".to_string(),
            support_level: "strategic-design".to_string(),
            security_domains: vec![
                "Source Code Security".to_string(),
                "Dependency Security".to_string(),
                "Configuration Security".to_string(),
                "Architecture Security".to_string(),
                "API Security".to_string(),
                "Infrastructure Security".to_string(),
            ],
            secret_categories: vec![
                "API Keys".to_string(),
                "JWT Secrets".to_string(),
                "Database Credentials".to_string(),
                "Cloud Credentials".to_string(),
                "Private Keys".to_string(),
                "Access Tokens".to_string(),
            ],
            dependency_sources: vec![
                "NVD".to_string(),
                "OSV".to_string(),
                "GitHub Advisories".to_string(),
            ],
            owasp_mappings: vec![
                "Broken Access Control".to_string(),
                "Cryptographic Failures".to_string(),
                "Injection".to_string(),
                "Security Misconfiguration".to_string(),
            ],
            risk_levels: vec![
                "Informational".to_string(),
                "Low".to_string(),
                "Medium".to_string(),
                "High".to_string(),
                "Critical".to_string(),
            ],
            report_types: vec![
                "Executive Summary".to_string(),
                "Developer Findings".to_string(),
                "Dependency Vulnerability Report".to_string(),
                "Configuration Risk Report".to_string(),
            ],
            runtime_crate_enabled: false,
            report_tables_enabled: false,
            secret_value_export_enabled: false,
            vulnerability_network_lookup_enabled: false,
            dashboard_enabled: false,
        }
    }

    pub fn git_readiness() -> GitReadiness {
        GitReadiness {
            enabled: false,
            status: "future-only".to_string(),
            support_level: "strategic-design".to_string(),
            data_sources: vec![
                "Commits".to_string(),
                "Branches".to_string(),
                "Tags".to_string(),
                "Authors".to_string(),
                "Changed Files".to_string(),
                "Directories".to_string(),
            ],
            analysis_domains: vec![
                "Repository Timeline".to_string(),
                "Contributor Intelligence".to_string(),
                "Ownership Mapping".to_string(),
                "Hotspot Analysis".to_string(),
                "Architecture Drift".to_string(),
                "Change Impact".to_string(),
                "Historical Architecture".to_string(),
            ],
            graph_node_types: vec![
                "Contributor".to_string(),
                "Commit".to_string(),
                "Branch".to_string(),
                "Tag".to_string(),
            ],
            relationship_types: vec![
                "Owns".to_string(),
                "Modified".to_string(),
                "Created".to_string(),
                "Reviewed".to_string(),
                "Depends On".to_string(),
            ],
            report_types: vec![
                "Timeline Report".to_string(),
                "Ownership Report".to_string(),
                "Hotspot Report".to_string(),
                "Architecture Drift Report".to_string(),
                "Historical Architecture Snapshot".to_string(),
            ],
            runtime_crate_enabled: false,
            history_mutation_enabled: false,
            repository_timeline_enabled: false,
            ownership_analysis_enabled: false,
            hotspot_analysis_enabled: false,
            drift_detection_enabled: false,
        }
    }

    pub fn backend_readiness() -> BackendReadiness {
        BackendReadiness {
            architecture: "React -> Tauri Commands -> App Core -> Rust Engines -> Storage"
                .to_string(),
            command_layer: "tauri-v2".to_string(),
            structured_errors: true,
            event_bus_enabled: true,
            storage_enabled: true,
            business_logic_in_rust: true,
            network_calls_enabled: false,
            service_crates: vec![
                "common".to_string(),
                "storage-engine".to_string(),
                "scanner-engine".to_string(),
                "parser-engine".to_string(),
                "graph-engine".to_string(),
                "docs-engine".to_string(),
                "uml-engine".to_string(),
                "export-engine".to_string(),
                "ai-engine".to_string(),
                "app-core".to_string(),
            ],
            future_crates: vec![
                "git-engine".to_string(),
                "security-engine".to_string(),
                "performance-engine".to_string(),
                "plugin-engine".to_string(),
                "mcp-server".to_string(),
            ],
        }
    }

    pub fn storage_readiness() -> StorageReadiness {
        let plan = StorageService::mvp_plan();
        StorageReadiness {
            primary_store: "SQLite".to_string(),
            operational_tables: plan
                .tables
                .iter()
                .filter(|table| {
                    !matches!(
                        **table,
                        "graph_snapshots"
                            | "ai_vector_snapshots"
                            | "chat_sessions"
                            | "chat_messages"
                    )
                })
                .map(|table| (*table).to_string())
                .collect(),
            snapshot_tables: vec![
                "graph_snapshots".to_string(),
                "ai_vector_snapshots".to_string(),
                "chat_sessions".to_string(),
                "chat_messages".to_string(),
            ],
            future_tables: vec![
                "modules".to_string(),
                "apis".to_string(),
                "models".to_string(),
                "reports".to_string(),
                "diagrams".to_string(),
                "ai_context_exports".to_string(),
                "plugin_registry".to_string(),
                "settings".to_string(),
                "user_profiles".to_string(),
                "repository_health".to_string(),
                "contributors".to_string(),
                "hotspots".to_string(),
                "ownership".to_string(),
                "graph_nodes".to_string(),
                "graph_edges".to_string(),
            ],
            vector_store_enabled: false,
            vector_store: "SQLite JSON snapshots for deterministic local embeddings".to_string(),
            search_index_enabled: false,
            search_index: "Tantivy is reserved for future full-text indexing".to_string(),
            graph_engine_enabled: true,
            graph_persistence: "Petgraph-style in-memory graph persisted as SQLite JSON snapshots"
                .to_string(),
            json_snapshots_enabled: true,
            cloud_tables_enabled: false,
        }
    }

    pub fn export_package(
        output_dir: &Path,
        scan: &ScanResult,
        documents: &[GeneratedDocument],
        diagrams: &[DiagramResult],
    ) -> DevAtlasResult<ExportPackage> {
        ExportService::build_knowledge_package(output_dir, scan, documents, diagrams)
    }

    pub fn storage_plan() -> StoragePlan {
        StorageService::mvp_plan()
    }

    pub fn open_storage(path: &Path) -> DevAtlasResult<SqliteStorage> {
        SqliteStorage::open(path)
    }

    pub fn open_memory_storage() -> DevAtlasResult<SqliteStorage> {
        SqliteStorage::open_in_memory()
    }
}

fn snapshot_to_embedding_result(snapshot: &AiVectorSnapshot) -> EmbeddingBuildResult {
    EmbeddingBuildResult {
        embeddings: snapshot
            .embeddings
            .iter()
            .map(|embedding| EmbeddingVector {
                id: embedding.id.clone(),
                chunk_id: embedding.chunk_id.clone(),
                path: embedding.path.clone(),
                dimensions: embedding.dimensions,
                model: embedding.model.clone(),
                values: embedding.values.clone(),
            })
            .collect(),
        dimensions: snapshot.dimensions,
        model: snapshot.model.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{AppService, EventBus, RepositoryMemoryInput};
    use devatlas_ai_engine::{ContextBuildResult, EmbeddingBuildResult, VectorStoreBuildResult};
    use devatlas_common::{
        ChatMessageRole, DiagramFormat, DiagramId, DiagramResult, DiagramType, DocumentId,
        DocumentType, DomainEvent, DomainEventPayload, GeneratedDocument, GraphEdge, GraphNode,
        KnowledgeGraph, RepositoryFile, ScanId, ScanResult, ScanStatus, Technology,
        TechnologyCategory,
    };

    #[test]
    fn exposes_mvp_storage_plan() {
        assert!(AppService::storage_plan()
            .tables
            .contains(&"graph_snapshots"));
    }

    #[test]
    fn exposes_storage_readiness_guardrails() {
        let readiness = AppService::storage_readiness();

        assert_eq!(readiness.primary_store, "SQLite");
        assert!(readiness
            .operational_tables
            .contains(&"repositories".to_string()));
        assert!(readiness
            .snapshot_tables
            .contains(&"ai_vector_snapshots".to_string()));
        assert!(readiness
            .future_tables
            .contains(&"plugin_registry".to_string()));
        assert!(!readiness.vector_store_enabled);
        assert!(!readiness.search_index_enabled);
        assert!(readiness.graph_engine_enabled);
        assert!(readiness.json_snapshots_enabled);
        assert!(!readiness.cloud_tables_enabled);
    }

    #[test]
    fn opens_memory_storage() {
        let storage = AppService::open_memory_storage().unwrap();
        assert_eq!(storage.list_repositories().unwrap().len(), 0);
    }

    #[test]
    fn publishes_domain_events() {
        let bus = EventBus::default();
        let mut receiver = bus.subscribe();
        bus.publish(DomainEvent::new(
            "repo-1",
            DomainEventPayload::RepositoryOpened {
                repository_id: "repo-1".to_string(),
                path: "sample".to_string(),
            },
        ));
        let event = receiver.try_recv().unwrap();
        assert_eq!(event.event_type, "Repository.RepositoryOpened");
    }

    #[test]
    fn builds_repository_memory_from_current_state() {
        let repository = AppService::open_repository(".").unwrap();
        let scan = ScanResult {
            scan_id: ScanId("scan-1".to_string()),
            repository_id: repository.id.clone(),
            status: ScanStatus::Completed,
            files_count: 2,
            folders_count: 1,
            technologies: vec![Technology {
                category: TechnologyCategory::Language,
                name: "Rust".to_string(),
                version: None,
            }],
            files: vec![RepositoryFile {
                path: "src/lib.rs".to_string(),
                extension: Some("rs".to_string()),
                size_bytes: 128,
            }],
            duration_ms: 12,
        };
        let graph = KnowledgeGraph {
            nodes: vec![GraphNode {
                id: "node-1".to_string(),
                node_type: "file".to_string(),
                name: "lib.rs".to_string(),
            }],
            edges: vec![GraphEdge {
                id: "edge-1".to_string(),
                source: "node-1".to_string(),
                target: "node-1".to_string(),
                edge_type: "self".to_string(),
            }],
        };
        let documents = vec![GeneratedDocument {
            id: DocumentId("doc-1".to_string()),
            path: "README.md".to_string(),
            document_type: DocumentType::Readme,
            content: "docs".to_string(),
        }];
        let diagrams = vec![DiagramResult {
            id: DiagramId("diagram-1".to_string()),
            path: "diagram.puml".to_string(),
            diagram_type: DiagramType::Component,
            format: DiagramFormat::PlantUml,
            content: "@startuml\n@enduml\n".to_string(),
        }];
        let context = ContextBuildResult {
            chunks: Vec::new(),
            skipped_files: Vec::new(),
        };
        let embeddings = EmbeddingBuildResult {
            embeddings: Vec::new(),
            dimensions: 128,
            model: "devatlas-local-hash-v1".to_string(),
        };
        let recent_questions = vec!["How does scanning work?".to_string()];

        let memory = AppService::build_repository_memory(RepositoryMemoryInput {
            repository: &repository,
            scan: Some(&scan),
            graph: Some(&graph),
            documents: &documents,
            diagrams: &diagrams,
            export_package: None,
            ai_context: Some(&context),
            ai_embeddings: Some(&embeddings),
            recent_questions: &recent_questions,
        });

        assert_eq!(memory.repository_id, repository.id.0);
        assert_eq!(memory.scan_id, Some("scan-1".to_string()));
        assert_eq!(memory.files_count, 2);
        assert_eq!(memory.graph_nodes, 1);
        assert_eq!(memory.document_count, 1);
        assert_eq!(memory.diagram_count, 1);
        assert_eq!(memory.ai_model, Some("devatlas-local-hash-v1".to_string()));
        assert_eq!(memory.recent_questions, recent_questions);
    }

    #[test]
    fn creates_graph_snapshot_metadata() {
        let repository = AppService::open_repository(".").unwrap();
        let graph = KnowledgeGraph {
            nodes: vec![GraphNode {
                id: "node-1".to_string(),
                node_type: "file".to_string(),
                name: "lib.rs".to_string(),
            }],
            edges: Vec::new(),
        };

        let snapshot = AppService::create_graph_snapshot(&repository, None, &graph);

        assert!(snapshot.id.0.starts_with("graph-snapshot-"));
        assert_eq!(snapshot.repository_id, repository.id);
        assert_eq!(snapshot.scan_id, None);
        assert_eq!(snapshot.node_count, 1);
        assert_eq!(snapshot.edge_count, 0);
    }

    #[test]
    fn creates_ai_vector_snapshot_metadata() {
        let repository = AppService::open_repository(".").unwrap();
        let vector_store = VectorStoreBuildResult {
            embeddings: Vec::new(),
            embedding_count: 0,
            dimensions: 128,
            model: "devatlas-local-hash-v1".to_string(),
        };

        let snapshot = AppService::create_ai_vector_snapshot(&repository, None, &vector_store);

        assert!(snapshot.id.0.starts_with("ai-vector-snapshot-"));
        assert_eq!(snapshot.repository_id, repository.id);
        assert_eq!(snapshot.scan_id, None);
        assert_eq!(snapshot.embedding_count, 0);
        assert_eq!(snapshot.dimensions, 128);
        assert_eq!(snapshot.model, "devatlas-local-hash-v1");
    }

    #[test]
    fn creates_chat_session_and_messages() {
        let repository = AppService::open_repository(".").unwrap();
        let session = AppService::create_chat_session(
            &repository,
            Some("  How does scanning work?  ".to_string()),
        );
        let message = AppService::create_chat_message(
            &repository,
            &session,
            ChatMessageRole::Assistant,
            "Scanning walks repository files.",
            Some("devatlas-local-grounded-v1".to_string()),
            2,
        );

        assert!(session.id.0.starts_with("chat-session-"));
        assert_eq!(session.repository_id, repository.id);
        assert_eq!(session.title, "How does scanning work?");
        assert!(message.id.0.starts_with("chat-message-"));
        assert_eq!(message.session_id, session.id);
        assert_eq!(message.role, ChatMessageRole::Assistant);
        assert_eq!(message.citation_count, 2);
    }

    #[test]
    fn exposes_cloud_readiness_without_enabling_cloud_runtime() {
        let readiness = AppService::cloud_readiness();

        assert!(!readiness.enabled);
        assert_eq!(readiness.status, "future-only");
        assert!(readiness.tenant_types.contains(&"Organization".to_string()));
        assert!(readiness.sync_modes.contains(&"Manual".to_string()));
        assert!(!readiness.network_required);
        assert!(!readiness.cloud_tables_enabled);
    }

    #[test]
    fn exposes_mcp_readiness_without_enabling_mcp_runtime() {
        let readiness = AppService::mcp_readiness();

        assert!(!readiness.enabled);
        assert_eq!(readiness.status, "future-only");
        assert!(readiness.transports.contains(&"STDIO".to_string()));
        assert!(readiness
            .resources
            .contains(&"repository://current".to_string()));
        assert!(readiness.tools.contains(&"ask_repository".to_string()));
        assert!(readiness
            .prompts
            .contains(&"Architecture Review".to_string()));
        assert!(!readiness.authentication_required);
        assert!(!readiness.runtime_crate_enabled);
    }

    #[test]
    fn exposes_plugin_readiness_without_enabling_plugin_runtime() {
        let readiness = AppService::plugin_readiness();

        assert!(!readiness.enabled);
        assert_eq!(readiness.status, "future-only");
        assert!(readiness.categories.contains(&"Scanner".to_string()));
        assert!(readiness.lifecycle.contains(&"Validate".to_string()));
        assert!(readiness
            .permissions
            .contains(&"Read Repository".to_string()));
        assert!(readiness
            .manifest_required_fields
            .contains(&"engineVersion".to_string()));
        assert!(readiness.sandbox_required);
        assert!(!readiness.marketplace_enabled);
        assert!(!readiness.runtime_crate_enabled);
        assert!(!readiness.registry_table_enabled);
        assert!(!readiness.network_access_enabled);
    }

    #[test]
    fn exposes_performance_readiness_without_enabling_performance_runtime() {
        let readiness = AppService::performance_readiness();

        assert!(!readiness.enabled);
        assert_eq!(readiness.status, "future-only");
        assert!(readiness
            .analysis_domains
            .contains(&"Code Complexity".to_string()));
        assert!(readiness
            .metrics
            .contains(&"Cyclomatic Complexity".to_string()));
        assert!(readiness.risk_levels.contains(&"Critical".to_string()));
        assert!(readiness
            .thresholds
            .contains(&"High Function: 100+ lines".to_string()));
        assert!(readiness
            .report_types
            .contains(&"Optimization Recommendations".to_string()));
        assert!(!readiness.runtime_crate_enabled);
        assert!(!readiness.report_tables_enabled);
        assert!(!readiness.profiling_enabled);
        assert!(!readiness.recommendations_enabled);
    }

    #[test]
    fn exposes_security_readiness_without_enabling_security_runtime() {
        let readiness = AppService::security_readiness();

        assert!(!readiness.enabled);
        assert_eq!(readiness.status, "future-only");
        assert!(readiness
            .security_domains
            .contains(&"Dependency Security".to_string()));
        assert!(readiness
            .secret_categories
            .contains(&"JWT Secrets".to_string()));
        assert!(readiness.dependency_sources.contains(&"OSV".to_string()));
        assert!(readiness
            .owasp_mappings
            .contains(&"Broken Access Control".to_string()));
        assert!(readiness.risk_levels.contains(&"Critical".to_string()));
        assert!(readiness
            .report_types
            .contains(&"Developer Findings".to_string()));
        assert!(!readiness.runtime_crate_enabled);
        assert!(!readiness.report_tables_enabled);
        assert!(!readiness.secret_value_export_enabled);
        assert!(!readiness.vulnerability_network_lookup_enabled);
        assert!(!readiness.dashboard_enabled);
    }

    #[test]
    fn exposes_git_readiness_without_enabling_git_runtime() {
        let readiness = AppService::git_readiness();

        assert!(!readiness.enabled);
        assert_eq!(readiness.status, "future-only");
        assert!(readiness.data_sources.contains(&"Commits".to_string()));
        assert!(readiness
            .analysis_domains
            .contains(&"Ownership Mapping".to_string()));
        assert!(readiness.graph_node_types.contains(&"Commit".to_string()));
        assert!(readiness
            .relationship_types
            .contains(&"Modified".to_string()));
        assert!(readiness
            .report_types
            .contains(&"Architecture Drift Report".to_string()));
        assert!(!readiness.runtime_crate_enabled);
        assert!(!readiness.history_mutation_enabled);
        assert!(!readiness.repository_timeline_enabled);
        assert!(!readiness.ownership_analysis_enabled);
        assert!(!readiness.hotspot_analysis_enabled);
        assert!(!readiness.drift_detection_enabled);
    }

    #[test]
    fn exposes_backend_readiness_contract() {
        let readiness = AppService::backend_readiness();

        assert_eq!(readiness.command_layer, "tauri-v2");
        assert!(readiness.structured_errors);
        assert!(readiness.event_bus_enabled);
        assert!(readiness.storage_enabled);
        assert!(readiness.business_logic_in_rust);
        assert!(!readiness.network_calls_enabled);
        assert!(readiness.service_crates.contains(&"app-core".to_string()));
        assert!(readiness.future_crates.contains(&"mcp-server".to_string()));
    }
}
