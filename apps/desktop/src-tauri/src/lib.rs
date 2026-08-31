use devatlas_ai_engine::{
    ChatResponse, ContextBuildResult, ContextBundle, EmbeddingBuildResult, EmbeddingVector,
    RetrievalResult, SkippedContextFileReason, VectorStoreBuildResult,
};
use devatlas_app_core::{AppService, EventBus, RepositoryMemoryInput, SqliteStorage};
use devatlas_common::{
    AiVectorEmbedding, AiVectorSnapshot, BackendReadiness, ChatMessage, ChatMessageRole,
    ChatSession, CloudReadiness, DevAtlasError, DiagramFormat, DiagramResult, DiagramType,
    DocumentType, DomainEvent, DomainEventPayload, ExportPackage, GeneratedDocument, GitReadiness,
    GraphEdge, GraphNode, GraphSnapshot, KnowledgeGraph, McpReadiness, PerformanceReadiness,
    PluginReadiness, Repository, RepositoryMemory, RepositoryMemoryTechnology, ScanResult,
    SecurityReadiness, StorageReadiness, TechnologyCategory,
};
use devatlas_scanner_engine::ScanOptions;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

const DOMAIN_EVENT_NAME: &str = "devatlas://domain-event";

struct AppState {
    repository: Mutex<Option<Repository>>,
    scan: Mutex<Option<ScanResult>>,
    graph: Mutex<Option<KnowledgeGraph>>,
    documents: Mutex<Vec<GeneratedDocument>>,
    diagrams: Mutex<Vec<DiagramResult>>,
    export_package: Mutex<Option<ExportPackage>>,
    ai_context: Mutex<Option<ContextBuildResult>>,
    ai_embeddings: Mutex<Option<EmbeddingBuildResult>>,
    ai_vector_store: Mutex<Option<VectorStoreBuildResult>>,
    ai_questions: Mutex<Vec<String>>,
    event_bus: EventBus,
    storage: Mutex<SqliteStorage>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandError {
    code: String,
    message: String,
}

impl CommandError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<DevAtlasError> for CommandError {
    fn from(error: DevAtlasError) -> Self {
        Self::new(error.code, error.message)
    }
}

type CommandResult<T> = Result<T, CommandError>;

impl AppState {
    fn new(storage: SqliteStorage) -> Self {
        Self {
            repository: Mutex::new(None),
            scan: Mutex::new(None),
            graph: Mutex::new(None),
            documents: Mutex::new(Vec::new()),
            diagrams: Mutex::new(Vec::new()),
            export_package: Mutex::new(None),
            ai_context: Mutex::new(None),
            ai_embeddings: Mutex::new(None),
            ai_vector_store: Mutex::new(None),
            ai_questions: Mutex::new(Vec::new()),
            event_bus: EventBus::default(),
            storage: Mutex::new(storage),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryDto {
    repository_id: String,
    name: String,
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TechnologyDto {
    category: String,
    name: String,
    version: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileDto {
    path: String,
    extension: Option<String>,
    size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResultDto {
    scan_id: String,
    repository_id: String,
    files: usize,
    folders: usize,
    duration_ms: u128,
    technologies: Vec<TechnologyDto>,
    repository_files: Vec<FileDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphSummaryDto {
    node_count: usize,
    edge_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphNodeDto {
    id: String,
    node_type: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphEdgeDto {
    id: String,
    source: String,
    target: String,
    edge_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeGraphDto {
    nodes: Vec<GraphNodeDto>,
    edges: Vec<GraphEdgeDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphSnapshotSummaryDto {
    id: String,
    repository_id: String,
    scan_id: Option<String>,
    node_count: usize,
    edge_count: usize,
    created_at_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphSnapshotDto {
    id: String,
    repository_id: String,
    scan_id: Option<String>,
    graph: KnowledgeGraphDto,
    node_count: usize,
    edge_count: usize,
    created_at_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedDocumentDto {
    id: String,
    path: String,
    document_type: String,
    content: String,
    semantic_plan: DocumentationPlanDto,
    quality: DocumentationQualityDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentationPlanDto {
    audience: String,
    intent: String,
    sections: Vec<DocumentationSectionPlanDto>,
    evidence_sources: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentationSectionPlanDto {
    title: String,
    purpose: String,
    evidence_type: String,
    required_signals: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentationQualityDto {
    coverage_score: u8,
    semantic_score: u8,
    source_count: usize,
    symbol_count: usize,
    graph_edge_count: usize,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagramResultDto {
    id: String,
    path: String,
    diagram_type: String,
    format: String,
    content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportPackageDto {
    id: String,
    path: String,
    artifacts_dir: String,
    artifact_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiContextChunkDto {
    id: String,
    path: String,
    chunk_index: usize,
    start_line: usize,
    end_line: usize,
    content: String,
    char_count: usize,
    token_estimate: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkippedContextFileDto {
    path: String,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiContextResultDto {
    chunks: Vec<AiContextChunkDto>,
    skipped_files: Vec<SkippedContextFileDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiEmbeddingVectorDto {
    id: String,
    chunk_id: String,
    path: String,
    dimensions: usize,
    model: String,
    values: Vec<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiEmbeddingResultDto {
    embeddings: Vec<AiEmbeddingVectorDto>,
    dimensions: usize,
    model: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVectorStoreResultDto {
    embedding_count: usize,
    dimensions: usize,
    model: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVectorSnapshotSummaryDto {
    id: String,
    repository_id: String,
    scan_id: Option<String>,
    embedding_count: usize,
    dimensions: usize,
    model: String,
    created_at_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVectorSnapshotDto {
    id: String,
    repository_id: String,
    scan_id: Option<String>,
    embeddings: Vec<AiEmbeddingVectorDto>,
    embedding_count: usize,
    dimensions: usize,
    model: String,
    created_at_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiRetrievalMatchDto {
    chunk_id: String,
    path: String,
    start_line: usize,
    end_line: usize,
    content: String,
    score: f32,
    vector_score: f32,
    lexical_score: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiRetrievalResultDto {
    query: String,
    matches: Vec<AiRetrievalMatchDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiContextBundleSourceDto {
    source_id: String,
    chunk_id: String,
    path: String,
    start_line: usize,
    end_line: usize,
    score: f32,
    token_estimate: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiContextBundleDto {
    query: String,
    content: String,
    sources: Vec<AiContextBundleSourceDto>,
    token_estimate: usize,
    max_tokens: usize,
    truncated: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiChatCitationDto {
    source_id: String,
    path: String,
    start_line: usize,
    end_line: usize,
    score: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiChatResponseDto {
    question: String,
    answer: String,
    citations: Vec<AiChatCitationDto>,
    context: AiContextBundleDto,
    model: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatSessionDto {
    id: String,
    repository_id: String,
    title: String,
    created_at_ms: u128,
    updated_at_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatMessageDto {
    id: String,
    repository_id: String,
    session_id: String,
    role: String,
    content: String,
    model: Option<String>,
    citation_count: usize,
    created_at_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryMemoryTechnologyDto {
    category: String,
    name: String,
    version: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryMemoryDto {
    repository_id: String,
    repository_name: String,
    path: String,
    scan_id: Option<String>,
    files_count: usize,
    folders_count: usize,
    technologies: Vec<RepositoryMemoryTechnologyDto>,
    graph_nodes: usize,
    graph_edges: usize,
    document_count: usize,
    diagram_count: usize,
    last_export_path: Option<String>,
    ai_context_chunks: usize,
    ai_embedding_count: usize,
    ai_model: Option<String>,
    recent_questions: Vec<String>,
    updated_at_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudReadinessDto {
    enabled: bool,
    status: String,
    support_level: String,
    tenant_types: Vec<String>,
    sync_modes: Vec<String>,
    deployment_models: Vec<String>,
    network_required: bool,
    cloud_tables_enabled: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct McpReadinessDto {
    enabled: bool,
    status: String,
    support_level: String,
    transports: Vec<String>,
    resources: Vec<String>,
    tools: Vec<String>,
    prompts: Vec<String>,
    authentication_required: bool,
    runtime_crate_enabled: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginReadinessDto {
    enabled: bool,
    status: String,
    support_level: String,
    categories: Vec<String>,
    lifecycle: Vec<String>,
    permissions: Vec<String>,
    manifest_required_fields: Vec<String>,
    sandbox_required: bool,
    marketplace_enabled: bool,
    runtime_crate_enabled: bool,
    registry_table_enabled: bool,
    network_access_enabled: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PerformanceReadinessDto {
    enabled: bool,
    status: String,
    support_level: String,
    analysis_domains: Vec<String>,
    metrics: Vec<String>,
    risk_levels: Vec<String>,
    thresholds: Vec<String>,
    report_types: Vec<String>,
    runtime_crate_enabled: bool,
    report_tables_enabled: bool,
    profiling_enabled: bool,
    recommendations_enabled: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SecurityReadinessDto {
    enabled: bool,
    status: String,
    support_level: String,
    security_domains: Vec<String>,
    secret_categories: Vec<String>,
    dependency_sources: Vec<String>,
    owasp_mappings: Vec<String>,
    risk_levels: Vec<String>,
    report_types: Vec<String>,
    runtime_crate_enabled: bool,
    report_tables_enabled: bool,
    secret_value_export_enabled: bool,
    vulnerability_network_lookup_enabled: bool,
    dashboard_enabled: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitReadinessDto {
    enabled: bool,
    status: String,
    support_level: String,
    data_sources: Vec<String>,
    analysis_domains: Vec<String>,
    graph_node_types: Vec<String>,
    relationship_types: Vec<String>,
    report_types: Vec<String>,
    runtime_crate_enabled: bool,
    history_mutation_enabled: bool,
    repository_timeline_enabled: bool,
    ownership_analysis_enabled: bool,
    hotspot_analysis_enabled: bool,
    drift_detection_enabled: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendReadinessDto {
    architecture: String,
    command_layer: String,
    structured_errors: bool,
    event_bus_enabled: bool,
    storage_enabled: bool,
    business_logic_in_rust: bool,
    network_calls_enabled: bool,
    service_crates: Vec<String>,
    future_crates: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageReadinessDto {
    primary_store: String,
    operational_tables: Vec<String>,
    snapshot_tables: Vec<String>,
    future_tables: Vec<String>,
    vector_store_enabled: bool,
    vector_store: String,
    search_index_enabled: bool,
    search_index: String,
    graph_engine_enabled: bool,
    graph_persistence: String,
    json_snapshots_enabled: bool,
    cloud_tables_enabled: bool,
}

#[tauri::command]
fn list_repositories(state: tauri::State<'_, AppState>) -> CommandResult<Vec<RepositoryDto>> {
    let repositories = state
        .storage
        .lock()
        .map_err(lock_error)?
        .list_repositories()
        .map_err(CommandError::from)?;
    Ok(repositories
        .into_iter()
        .map(|repository| RepositoryDto {
            repository_id: repository.id,
            name: repository.name,
            path: repository.path,
        })
        .collect())
}

#[tauri::command]
fn open_repository(
    path: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<RepositoryDto> {
    let repository = AppService::open_repository(path).map_err(CommandError::from)?;
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_repository(&repository)
        .map_err(CommandError::from)?;
    let dto = repository_to_dto(&repository);
    publish_event(
        &state,
        DomainEvent::new(
            repository.id.0.clone(),
            DomainEventPayload::RepositoryOpened {
                repository_id: repository.id.0.clone(),
                path: repository.path.as_path().to_string_lossy().to_string(),
            },
        ),
    );
    *state.repository.lock().map_err(lock_error)? = Some(repository);
    Ok(dto)
}

#[tauri::command]
fn scan_repository(
    max_files: Option<usize>,
    include_paths: Option<Vec<String>>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<ScanResultDto> {
    let repository = state
        .repository
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error("repository.not_open", "Open a repository before scanning.")
        })?;
    let scan = AppService::scan_repository_with_options(
        &repository,
        &ScanOptions {
            max_files: max_files.filter(|value| *value > 0),
            include_paths: include_paths.unwrap_or_default(),
        },
    )
    .map_err(CommandError::from)?;
    let graph =
        AppService::build_graph_for_repository(&repository, &scan).map_err(CommandError::from)?;
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_scan(&scan)
        .map_err(CommandError::from)?;
    let dto = scan_to_dto(&scan);
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::ScanStarted {
                scan_id: scan.scan_id.0.clone(),
                repository_id: scan.repository_id.0.clone(),
            },
        ),
    );
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::ScanCompleted {
                scan_id: scan.scan_id.0.clone(),
                repository_id: scan.repository_id.0.clone(),
                duration_ms: scan.duration_ms,
            },
        ),
    );
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::GraphBuilt {
                repository_id: scan.repository_id.0.clone(),
                node_count: graph.nodes.len(),
                edge_count: graph.edges.len(),
            },
        ),
    );
    *state.scan.lock().map_err(lock_error)? = Some(scan);
    *state.graph.lock().map_err(lock_error)? = Some(graph);
    Ok(dto)
}

#[tauri::command]
fn list_repository_files(state: tauri::State<'_, AppState>) -> CommandResult<Vec<FileDto>> {
    let scan = current_or_new_scan(&state)?;
    Ok(scan
        .files
        .iter()
        .map(|file| FileDto {
            path: file.path.clone(),
            extension: file.extension.clone(),
            size_bytes: file.size_bytes,
        })
        .collect())
}

#[tauri::command]
fn detect_technologies(state: tauri::State<'_, AppState>) -> CommandResult<Vec<TechnologyDto>> {
    let scan = current_or_new_scan(&state)?;
    Ok(scan
        .technologies
        .iter()
        .map(|technology| TechnologyDto {
            category: category_to_string(&technology.category),
            name: technology.name.clone(),
            version: technology.version.clone(),
        })
        .collect())
}

#[tauri::command]
fn get_graph(state: tauri::State<'_, AppState>) -> CommandResult<KnowledgeGraphDto> {
    let scan = current_or_new_scan(&state)?;
    let graph = current_or_new_graph(&state, &scan)?;
    Ok(knowledge_graph_to_dto(&graph))
}

#[tauri::command]
fn build_graph(state: tauri::State<'_, AppState>) -> CommandResult<GraphSummaryDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before building the graph.",
            )
        })?;
    let graph =
        AppService::build_graph_for_repository(&repository, &scan).map_err(CommandError::from)?;
    let summary = GraphSummaryDto {
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
    };
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::GraphBuilt {
                repository_id: scan.repository_id.0.clone(),
                node_count: graph.nodes.len(),
                edge_count: graph.edges.len(),
            },
        ),
    );
    *state.graph.lock().map_err(lock_error)? = Some(graph);
    Ok(summary)
}

#[tauri::command]
fn save_graph_snapshot(
    state: tauri::State<'_, AppState>,
) -> CommandResult<GraphSnapshotSummaryDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before saving a graph snapshot.",
            )
        })?;
    let graph = current_or_new_graph(&state, &scan)?;
    let snapshot = AppService::create_graph_snapshot(&repository, Some(&scan), &graph);
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_graph_snapshot(&snapshot)
        .map_err(CommandError::from)?;
    publish_event(
        &state,
        DomainEvent::new(
            snapshot.repository_id.0.clone(),
            DomainEventPayload::GraphSnapshotSaved {
                repository_id: snapshot.repository_id.0.clone(),
                snapshot_id: snapshot.id.0.clone(),
                node_count: snapshot.node_count,
                edge_count: snapshot.edge_count,
            },
        ),
    );
    Ok(graph_snapshot_summary_to_dto(&snapshot))
}

#[tauri::command]
fn list_graph_snapshots(
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<GraphSnapshotSummaryDto>> {
    let repository = current_repository(&state)?;
    let snapshots = state
        .storage
        .lock()
        .map_err(lock_error)?
        .list_graph_snapshots(&repository.id.0)
        .map_err(CommandError::from)?;
    Ok(snapshots
        .into_iter()
        .map(|snapshot| GraphSnapshotSummaryDto {
            id: snapshot.id,
            repository_id: snapshot.repository_id,
            scan_id: snapshot.scan_id,
            node_count: snapshot.node_count,
            edge_count: snapshot.edge_count,
            created_at_ms: snapshot.created_at_ms,
        })
        .collect())
}

#[tauri::command]
fn load_graph_snapshot(
    snapshot_id: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<GraphSnapshotDto> {
    let snapshot = state
        .storage
        .lock()
        .map_err(lock_error)?
        .load_graph_snapshot(&snapshot_id)
        .map_err(CommandError::from)?;
    let dto = graph_snapshot_to_dto(&snapshot);
    *state.graph.lock().map_err(lock_error)? = Some(snapshot.graph);
    Ok(dto)
}

#[tauri::command]
fn generate_docs(state: tauri::State<'_, AppState>) -> CommandResult<Vec<GeneratedDocumentDto>> {
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before generating documentation.",
            )
        })?;
    let graph = current_or_new_graph(&state, &scan)?;
    let documents = AppService::generate_docs(&scan, &graph);
    let dto = documents
        .iter()
        .map(document_to_dto)
        .collect::<Vec<GeneratedDocumentDto>>();
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::DocumentationGenerated {
                repository_id: scan.repository_id.0.clone(),
                document_count: documents.len(),
            },
        ),
    );
    *state.documents.lock().map_err(lock_error)? = documents;
    Ok(dto)
}

#[tauri::command]
fn generate_diagrams(state: tauri::State<'_, AppState>) -> CommandResult<Vec<DiagramResultDto>> {
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before generating diagrams.",
            )
        })?;
    let graph = current_or_new_graph(&state, &scan)?;
    let diagrams = AppService::generate_diagrams(&scan, &graph);
    let dto = diagrams
        .iter()
        .map(diagram_to_dto)
        .collect::<Vec<DiagramResultDto>>();
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::DiagramGenerated {
                repository_id: scan.repository_id.0.clone(),
                diagram_count: diagrams.len(),
            },
        ),
    );
    *state.diagrams.lock().map_err(lock_error)? = diagrams;
    Ok(dto)
}

#[tauri::command]
fn export_package(
    output_dir: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<ExportPackageDto> {
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| command_error("scan.not_available", "Run a scan before exporting."))?;
    let graph = current_or_new_graph(&state, &scan)?;
    let documents = {
        let mut stored_documents = state.documents.lock().map_err(lock_error)?;
        if stored_documents.is_empty() {
            *stored_documents = AppService::generate_docs(&scan, &graph);
        }
        stored_documents.clone()
    };
    let diagrams = {
        let mut stored_diagrams = state.diagrams.lock().map_err(lock_error)?;
        if stored_diagrams.is_empty() {
            *stored_diagrams = AppService::generate_diagrams(&scan, &graph);
        }
        stored_diagrams.clone()
    };
    let package = AppService::export_package(
        std::path::Path::new(&output_dir),
        &scan,
        &documents,
        &diagrams,
    )
    .map_err(CommandError::from)?;
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_export(&scan.repository_id.0, &package)
        .map_err(CommandError::from)?;
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::ExportCompleted {
                repository_id: scan.repository_id.0.clone(),
                path: package.path.clone(),
            },
        ),
    );
    *state.export_package.lock().map_err(lock_error)? = Some(package.clone());
    Ok(export_to_dto(&package))
}

#[tauri::command]
fn build_ai_context(state: tauri::State<'_, AppState>) -> CommandResult<AiContextResultDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before building AI context.",
            )
        })?;
    let context = AppService::build_ai_context(&repository, &scan).map_err(CommandError::from)?;
    let dto = ai_context_to_dto(&context);
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiContextBuilt {
                repository_id: scan.repository_id.0.clone(),
                chunk_count: context.chunks.len(),
                skipped_file_count: context.skipped_files.len(),
            },
        ),
    );
    *state.ai_context.lock().map_err(lock_error)? = Some(context);
    Ok(dto)
}

#[tauri::command]
fn build_ai_embeddings(state: tauri::State<'_, AppState>) -> CommandResult<AiEmbeddingResultDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before building AI embeddings.",
            )
        })?;
    let embeddings =
        AppService::build_ai_embeddings(&repository, &scan).map_err(CommandError::from)?;
    let dto = ai_embeddings_to_dto(&embeddings);
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiEmbeddingsBuilt {
                repository_id: scan.repository_id.0.clone(),
                embedding_count: embeddings.embeddings.len(),
                dimensions: embeddings.dimensions,
                model: embeddings.model.clone(),
            },
        ),
    );
    *state.ai_embeddings.lock().map_err(lock_error)? = Some(embeddings);
    Ok(dto)
}

#[tauri::command]
fn build_ai_vector_store(
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiVectorStoreResultDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before building the AI vector store.",
            )
        })?;
    let vector_store =
        AppService::build_ai_vector_store(&repository, &scan).map_err(CommandError::from)?;
    let dto = ai_vector_store_to_dto(&vector_store);
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiVectorStoreBuilt {
                repository_id: scan.repository_id.0.clone(),
                embedding_count: vector_store.embedding_count,
                dimensions: vector_store.dimensions,
                model: vector_store.model.clone(),
            },
        ),
    );
    *state.ai_vector_store.lock().map_err(lock_error)? = Some(vector_store);
    Ok(dto)
}

#[tauri::command]
fn save_ai_vector_snapshot(
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiVectorSnapshotSummaryDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before saving an AI vector snapshot.",
            )
        })?;
    let vector_store = match state.ai_vector_store.lock().map_err(lock_error)?.clone() {
        Some(current_store) => current_store,
        None => {
            AppService::build_ai_vector_store(&repository, &scan).map_err(CommandError::from)?
        }
    };
    let snapshot = AppService::create_ai_vector_snapshot(&repository, Some(&scan), &vector_store);
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_ai_vector_snapshot(&snapshot)
        .map_err(CommandError::from)?;
    *state.ai_vector_store.lock().map_err(lock_error)? = Some(vector_store);
    publish_event(
        &state,
        DomainEvent::new(
            snapshot.repository_id.0.clone(),
            DomainEventPayload::AiVectorSnapshotSaved {
                repository_id: snapshot.repository_id.0.clone(),
                snapshot_id: snapshot.id.0.clone(),
                embedding_count: snapshot.embedding_count,
                dimensions: snapshot.dimensions,
                model: snapshot.model.clone(),
            },
        ),
    );
    Ok(ai_vector_snapshot_summary_to_dto(&snapshot))
}

#[tauri::command]
fn list_ai_vector_snapshots(
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<AiVectorSnapshotSummaryDto>> {
    let repository = current_repository(&state)?;
    let snapshots = state
        .storage
        .lock()
        .map_err(lock_error)?
        .list_ai_vector_snapshots(&repository.id.0)
        .map_err(CommandError::from)?;
    Ok(snapshots
        .into_iter()
        .map(|snapshot| AiVectorSnapshotSummaryDto {
            id: snapshot.id,
            repository_id: snapshot.repository_id,
            scan_id: snapshot.scan_id,
            embedding_count: snapshot.embedding_count,
            dimensions: snapshot.dimensions,
            model: snapshot.model,
            created_at_ms: snapshot.created_at_ms,
        })
        .collect())
}

#[tauri::command]
fn load_ai_vector_snapshot(
    snapshot_id: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiVectorSnapshotDto> {
    let snapshot = state
        .storage
        .lock()
        .map_err(lock_error)?
        .load_ai_vector_snapshot(&snapshot_id)
        .map_err(CommandError::from)?;
    let vector_store = ai_vector_snapshot_to_store(&snapshot);
    let dto = ai_vector_snapshot_to_dto(&snapshot);
    *state.ai_vector_store.lock().map_err(lock_error)? = Some(vector_store);
    Ok(dto)
}

#[tauri::command]
fn search_ai_context(
    query: String,
    limit: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiRetrievalResultDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before searching AI context.",
            )
        })?;
    let result = AppService::search_ai_context(&repository, &scan, query, limit)
        .map_err(CommandError::from)?;
    let dto = ai_retrieval_to_dto(&result);
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiRetrievalCompleted {
                repository_id: scan.repository_id.0.clone(),
                query: result.query.clone(),
                match_count: result.matches.len(),
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn search_ai_context_snapshot(
    snapshot_id: String,
    query: String,
    limit: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiRetrievalResultDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before searching saved AI context.",
            )
        })?;
    let snapshot = state
        .storage
        .lock()
        .map_err(lock_error)?
        .load_ai_vector_snapshot(&snapshot_id)
        .map_err(CommandError::from)?;
    let result =
        AppService::search_ai_context_snapshot(&repository, &scan, &snapshot, query, limit)
            .map_err(CommandError::from)?;
    let dto = ai_retrieval_to_dto(&result);
    *state.ai_vector_store.lock().map_err(lock_error)? =
        Some(ai_vector_snapshot_to_store(&snapshot));
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiPersistedRetrievalCompleted {
                repository_id: scan.repository_id.0.clone(),
                snapshot_id,
                query: result.query.clone(),
                match_count: result.matches.len(),
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn build_ai_context_bundle(
    query: String,
    limit: Option<usize>,
    max_tokens: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiContextBundleDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before building an AI context bundle.",
            )
        })?;
    let bundle = AppService::build_ai_context_bundle(&repository, &scan, query, limit, max_tokens)
        .map_err(CommandError::from)?;
    let dto = ai_context_bundle_to_dto(&bundle);
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiContextBundleBuilt {
                repository_id: scan.repository_id.0.clone(),
                query: bundle.query.clone(),
                source_count: bundle.sources.len(),
                token_estimate: bundle.token_estimate,
                truncated: bundle.truncated,
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn build_ai_context_bundle_snapshot(
    snapshot_id: String,
    query: String,
    limit: Option<usize>,
    max_tokens: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiContextBundleDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before building saved AI context.",
            )
        })?;
    let snapshot = state
        .storage
        .lock()
        .map_err(lock_error)?
        .load_ai_vector_snapshot(&snapshot_id)
        .map_err(CommandError::from)?;
    let bundle = AppService::build_ai_context_bundle_snapshot(
        &repository,
        &scan,
        &snapshot,
        query,
        limit,
        max_tokens,
    )
    .map_err(CommandError::from)?;
    let dto = ai_context_bundle_to_dto(&bundle);
    *state.ai_vector_store.lock().map_err(lock_error)? =
        Some(ai_vector_snapshot_to_store(&snapshot));
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiPersistedContextBundleBuilt {
                repository_id: scan.repository_id.0.clone(),
                snapshot_id,
                query: bundle.query.clone(),
                source_count: bundle.sources.len(),
                token_estimate: bundle.token_estimate,
                truncated: bundle.truncated,
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn ask_ai(
    question: String,
    limit: Option<usize>,
    max_context_tokens: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiChatResponseDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| command_error("scan.not_available", "Run a scan before asking AI."))?;
    let response = AppService::ask_ai(&repository, &scan, question, limit, max_context_tokens)
        .map_err(CommandError::from)?;
    let dto = ai_chat_response_to_dto(&response);
    remember_ai_question(&state, response.question.clone())?;
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiChatResponded {
                repository_id: scan.repository_id.0.clone(),
                question: response.question.clone(),
                citation_count: response.citations.len(),
                model: response.model.clone(),
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn ask_ai_snapshot(
    snapshot_id: String,
    question: String,
    limit: Option<usize>,
    max_context_tokens: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiChatResponseDto> {
    let repository = current_repository(&state)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before asking saved AI context.",
            )
        })?;
    let snapshot = state
        .storage
        .lock()
        .map_err(lock_error)?
        .load_ai_vector_snapshot(&snapshot_id)
        .map_err(CommandError::from)?;
    let response = AppService::ask_ai_snapshot(
        &repository,
        &scan,
        &snapshot,
        question,
        limit,
        max_context_tokens,
    )
    .map_err(CommandError::from)?;
    let dto = ai_chat_response_to_dto(&response);
    remember_ai_question(&state, response.question.clone())?;
    *state.ai_vector_store.lock().map_err(lock_error)? =
        Some(ai_vector_snapshot_to_store(&snapshot));
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiPersistedChatResponded {
                repository_id: scan.repository_id.0.clone(),
                snapshot_id,
                question: response.question.clone(),
                citation_count: response.citations.len(),
                model: response.model.clone(),
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn start_chat_session(
    title: Option<String>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<ChatSessionDto> {
    let repository = current_repository(&state)?;
    let session = AppService::create_chat_session(&repository, title);
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_chat_session(&session)
        .map_err(CommandError::from)?;
    publish_event(
        &state,
        DomainEvent::new(
            repository.id.0.clone(),
            DomainEventPayload::AiChatSessionStarted {
                repository_id: repository.id.0.clone(),
                session_id: session.id.0.clone(),
                title: session.title.clone(),
            },
        ),
    );
    Ok(chat_session_to_dto(&session))
}

#[tauri::command]
fn list_chat_sessions(state: tauri::State<'_, AppState>) -> CommandResult<Vec<ChatSessionDto>> {
    let repository = current_repository(&state)?;
    let sessions = state
        .storage
        .lock()
        .map_err(lock_error)?
        .list_chat_sessions(&repository.id.0)
        .map_err(CommandError::from)?;
    Ok(sessions
        .iter()
        .map(chat_session_to_dto)
        .collect::<Vec<ChatSessionDto>>())
}

#[tauri::command]
fn list_chat_messages(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<ChatMessageDto>> {
    let repository = current_repository(&state)?;
    let session = load_current_chat_session(&state, &repository, &session_id)?;
    let messages = state
        .storage
        .lock()
        .map_err(lock_error)?
        .list_chat_messages(&session.id.0)
        .map_err(CommandError::from)?;
    Ok(messages
        .iter()
        .map(chat_message_to_dto)
        .collect::<Vec<ChatMessageDto>>())
}

#[tauri::command]
fn ask_ai_in_session(
    session_id: String,
    question: String,
    limit: Option<usize>,
    max_context_tokens: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiChatResponseDto> {
    let repository = current_repository(&state)?;
    let session = load_current_chat_session(&state, &repository, &session_id)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| command_error("scan.not_available", "Run a scan before asking AI."))?;
    let user_message = AppService::create_chat_message(
        &repository,
        &session,
        ChatMessageRole::User,
        question.clone(),
        None,
        0,
    );
    save_chat_message(&state, &user_message)?;
    let response = AppService::ask_ai(&repository, &scan, question, limit, max_context_tokens)
        .map_err(CommandError::from)?;
    let assistant_message = AppService::create_chat_message(
        &repository,
        &session,
        ChatMessageRole::Assistant,
        response.answer.clone(),
        Some(response.model.clone()),
        response.citations.len(),
    );
    save_chat_message(&state, &assistant_message)?;
    let dto = ai_chat_response_to_dto(&response);
    remember_ai_question(&state, response.question.clone())?;
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiChatResponded {
                repository_id: scan.repository_id.0.clone(),
                question: response.question.clone(),
                citation_count: response.citations.len(),
                model: response.model.clone(),
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn ask_ai_snapshot_in_session(
    session_id: String,
    snapshot_id: String,
    question: String,
    limit: Option<usize>,
    max_context_tokens: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> CommandResult<AiChatResponseDto> {
    let repository = current_repository(&state)?;
    let session = load_current_chat_session(&state, &repository, &session_id)?;
    let scan = state
        .scan
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "scan.not_available",
                "Run a scan before asking saved AI context.",
            )
        })?;
    let snapshot = state
        .storage
        .lock()
        .map_err(lock_error)?
        .load_ai_vector_snapshot(&snapshot_id)
        .map_err(CommandError::from)?;
    let user_message = AppService::create_chat_message(
        &repository,
        &session,
        ChatMessageRole::User,
        question.clone(),
        None,
        0,
    );
    save_chat_message(&state, &user_message)?;
    let response = AppService::ask_ai_snapshot(
        &repository,
        &scan,
        &snapshot,
        question,
        limit,
        max_context_tokens,
    )
    .map_err(CommandError::from)?;
    let assistant_message = AppService::create_chat_message(
        &repository,
        &session,
        ChatMessageRole::Assistant,
        response.answer.clone(),
        Some(response.model.clone()),
        response.citations.len(),
    );
    save_chat_message(&state, &assistant_message)?;
    let dto = ai_chat_response_to_dto(&response);
    remember_ai_question(&state, response.question.clone())?;
    *state.ai_vector_store.lock().map_err(lock_error)? =
        Some(ai_vector_snapshot_to_store(&snapshot));
    publish_event(
        &state,
        DomainEvent::new(
            scan.repository_id.0.clone(),
            DomainEventPayload::AiPersistedChatResponded {
                repository_id: scan.repository_id.0.clone(),
                snapshot_id,
                question: response.question.clone(),
                citation_count: response.citations.len(),
                model: response.model.clone(),
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn build_repository_memory(
    state: tauri::State<'_, AppState>,
) -> CommandResult<RepositoryMemoryDto> {
    let repository = current_repository(&state)?;
    let scan = state.scan.lock().map_err(lock_error)?.clone();
    let graph = state.graph.lock().map_err(lock_error)?.clone();
    let documents = state.documents.lock().map_err(lock_error)?.clone();
    let diagrams = state.diagrams.lock().map_err(lock_error)?.clone();
    let export_package = state.export_package.lock().map_err(lock_error)?.clone();
    let ai_context = state.ai_context.lock().map_err(lock_error)?.clone();
    let ai_embeddings = state.ai_embeddings.lock().map_err(lock_error)?.clone();
    let recent_questions = state.ai_questions.lock().map_err(lock_error)?.clone();

    let memory = AppService::build_repository_memory(RepositoryMemoryInput {
        repository: &repository,
        scan: scan.as_ref(),
        graph: graph.as_ref(),
        documents: &documents,
        diagrams: &diagrams,
        export_package: export_package.as_ref(),
        ai_context: ai_context.as_ref(),
        ai_embeddings: ai_embeddings.as_ref(),
        recent_questions: &recent_questions,
    });
    let dto = repository_memory_to_dto(&memory);
    publish_event(
        &state,
        DomainEvent::new(
            memory.repository_id.clone(),
            DomainEventPayload::RepositoryMemoryBuilt {
                repository_id: memory.repository_id,
                files_count: memory.files_count,
                graph_nodes: memory.graph_nodes,
                ai_context_chunks: memory.ai_context_chunks,
            },
        ),
    );
    Ok(dto)
}

#[tauri::command]
fn get_cloud_status() -> CloudReadinessDto {
    cloud_readiness_to_dto(&AppService::cloud_readiness())
}

#[tauri::command]
fn get_mcp_status() -> McpReadinessDto {
    mcp_readiness_to_dto(&AppService::mcp_readiness())
}

#[tauri::command]
fn get_plugin_status() -> PluginReadinessDto {
    plugin_readiness_to_dto(&AppService::plugin_readiness())
}

#[tauri::command]
fn get_performance_status() -> PerformanceReadinessDto {
    performance_readiness_to_dto(&AppService::performance_readiness())
}

#[tauri::command]
fn get_security_status() -> SecurityReadinessDto {
    security_readiness_to_dto(&AppService::security_readiness())
}

#[tauri::command]
fn get_git_status() -> GitReadinessDto {
    git_readiness_to_dto(&AppService::git_readiness())
}

#[tauri::command]
fn get_backend_status() -> BackendReadinessDto {
    backend_readiness_to_dto(&AppService::backend_readiness())
}

#[tauri::command]
fn get_storage_status() -> StorageReadinessDto {
    storage_readiness_to_dto(&AppService::storage_readiness())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let storage = AppService::open_storage(&app_data_dir.join("devatlas.db"))?;
            let state = AppState::new(storage);
            let mut receiver = state.event_bus.subscribe();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    match receiver.recv().await {
                        Ok(event) => {
                            let _ = app_handle.emit(DOMAIN_EVENT_NAME, event);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_repositories,
            open_repository,
            scan_repository,
            list_repository_files,
            detect_technologies,
            build_graph,
            get_graph,
            save_graph_snapshot,
            list_graph_snapshots,
            load_graph_snapshot,
            generate_docs,
            generate_diagrams,
            export_package,
            build_ai_context,
            build_ai_embeddings,
            build_ai_vector_store,
            save_ai_vector_snapshot,
            list_ai_vector_snapshots,
            load_ai_vector_snapshot,
            search_ai_context,
            search_ai_context_snapshot,
            build_ai_context_bundle,
            build_ai_context_bundle_snapshot,
            ask_ai,
            ask_ai_snapshot,
            start_chat_session,
            list_chat_sessions,
            list_chat_messages,
            ask_ai_in_session,
            ask_ai_snapshot_in_session,
            build_repository_memory,
            get_cloud_status,
            get_mcp_status,
            get_plugin_status,
            get_performance_status,
            get_security_status,
            get_git_status,
            get_backend_status,
            get_storage_status
        ])
        .run(tauri::generate_context!())
        .expect("failed to run DevAtlas");
}

fn current_or_new_scan(state: &tauri::State<'_, AppState>) -> CommandResult<ScanResult> {
    if let Some(scan) = state.scan.lock().map_err(lock_error)?.clone() {
        return Ok(scan);
    }

    let repository = current_repository(state)?;
    let scan = AppService::scan_repository(&repository).map_err(CommandError::from)?;
    let graph =
        AppService::build_graph_for_repository(&repository, &scan).map_err(CommandError::from)?;
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_scan(&scan)
        .map_err(CommandError::from)?;
    *state.scan.lock().map_err(lock_error)? = Some(scan.clone());
    *state.graph.lock().map_err(lock_error)? = Some(graph);
    Ok(scan)
}

fn current_or_new_graph(
    state: &tauri::State<'_, AppState>,
    scan: &ScanResult,
) -> CommandResult<KnowledgeGraph> {
    if let Some(graph) = state.graph.lock().map_err(lock_error)?.clone() {
        return Ok(graph);
    }
    let repository = current_repository(state)?;
    let graph =
        AppService::build_graph_for_repository(&repository, scan).map_err(CommandError::from)?;
    *state.graph.lock().map_err(lock_error)? = Some(graph.clone());
    Ok(graph)
}

fn current_repository(state: &tauri::State<'_, AppState>) -> CommandResult<Repository> {
    state
        .repository
        .lock()
        .map_err(lock_error)?
        .clone()
        .ok_or_else(|| {
            command_error(
                "repository.not_open",
                "Open a repository before reading repository data.",
            )
        })
}

fn load_current_chat_session(
    state: &tauri::State<'_, AppState>,
    repository: &Repository,
    session_id: &str,
) -> CommandResult<ChatSession> {
    let session = state
        .storage
        .lock()
        .map_err(lock_error)?
        .load_chat_session(session_id)
        .map_err(CommandError::from)?;
    if session.repository_id != repository.id {
        return Err(command_error(
            "chat_session.repository_mismatch",
            "Chat session does not belong to the opened repository.",
        ));
    }
    Ok(session)
}

fn save_chat_message(
    state: &tauri::State<'_, AppState>,
    message: &ChatMessage,
) -> CommandResult<()> {
    state
        .storage
        .lock()
        .map_err(lock_error)?
        .save_chat_message(message)
        .map_err(CommandError::from)?;
    publish_event(
        state,
        DomainEvent::new(
            message.repository_id.0.clone(),
            DomainEventPayload::AiChatMessageSaved {
                repository_id: message.repository_id.0.clone(),
                session_id: message.session_id.0.clone(),
                message_id: message.id.0.clone(),
                role: message.role.as_str().to_string(),
            },
        ),
    );
    Ok(())
}

fn repository_to_dto(repository: &Repository) -> RepositoryDto {
    RepositoryDto {
        repository_id: repository.id.0.clone(),
        name: repository.name.clone(),
        path: repository.path.as_path().to_string_lossy().to_string(),
    }
}

fn scan_to_dto(scan: &ScanResult) -> ScanResultDto {
    ScanResultDto {
        scan_id: scan.scan_id.0.clone(),
        repository_id: scan.repository_id.0.clone(),
        files: scan.files_count,
        folders: scan.folders_count,
        duration_ms: scan.duration_ms,
        technologies: scan
            .technologies
            .iter()
            .map(|technology| TechnologyDto {
                category: category_to_string(&technology.category),
                name: technology.name.clone(),
                version: technology.version.clone(),
            })
            .collect(),
        repository_files: scan
            .files
            .iter()
            .map(|file| FileDto {
                path: file.path.clone(),
                extension: file.extension.clone(),
                size_bytes: file.size_bytes,
            })
            .collect(),
    }
}

fn document_to_dto(document: &GeneratedDocument) -> GeneratedDocumentDto {
    GeneratedDocumentDto {
        id: document.id.0.clone(),
        path: document.path.clone(),
        document_type: document_type_to_string(&document.document_type),
        content: document.content.clone(),
        semantic_plan: DocumentationPlanDto {
            audience: document.semantic_plan.audience.clone(),
            intent: document.semantic_plan.intent.clone(),
            sections: document
                .semantic_plan
                .sections
                .iter()
                .map(|section| DocumentationSectionPlanDto {
                    title: section.title.clone(),
                    purpose: section.purpose.clone(),
                    evidence_type: section.evidence_type.clone(),
                    required_signals: section.required_signals.clone(),
                })
                .collect(),
            evidence_sources: document.semantic_plan.evidence_sources.clone(),
        },
        quality: DocumentationQualityDto {
            coverage_score: document.quality.coverage_score,
            semantic_score: document.quality.semantic_score,
            source_count: document.quality.source_count,
            symbol_count: document.quality.symbol_count,
            graph_edge_count: document.quality.graph_edge_count,
            warnings: document.quality.warnings.clone(),
        },
    }
}

fn diagram_to_dto(diagram: &DiagramResult) -> DiagramResultDto {
    DiagramResultDto {
        id: diagram.id.0.clone(),
        path: diagram.path.clone(),
        diagram_type: diagram_type_to_string(&diagram.diagram_type),
        format: diagram_format_to_string(&diagram.format),
        content: diagram.content.clone(),
    }
}

fn export_to_dto(package: &ExportPackage) -> ExportPackageDto {
    ExportPackageDto {
        id: package.id.0.clone(),
        path: package.path.clone(),
        artifacts_dir: package.artifacts_dir.clone(),
        artifact_count: package.artifact_count,
    }
}

fn graph_snapshot_summary_to_dto(snapshot: &GraphSnapshot) -> GraphSnapshotSummaryDto {
    GraphSnapshotSummaryDto {
        id: snapshot.id.0.clone(),
        repository_id: snapshot.repository_id.0.clone(),
        scan_id: snapshot.scan_id.as_ref().map(|scan_id| scan_id.0.clone()),
        node_count: snapshot.node_count,
        edge_count: snapshot.edge_count,
        created_at_ms: snapshot.created_at_ms,
    }
}

fn graph_snapshot_to_dto(snapshot: &GraphSnapshot) -> GraphSnapshotDto {
    GraphSnapshotDto {
        id: snapshot.id.0.clone(),
        repository_id: snapshot.repository_id.0.clone(),
        scan_id: snapshot.scan_id.as_ref().map(|scan_id| scan_id.0.clone()),
        graph: knowledge_graph_to_dto(&snapshot.graph),
        node_count: snapshot.node_count,
        edge_count: snapshot.edge_count,
        created_at_ms: snapshot.created_at_ms,
    }
}

fn knowledge_graph_to_dto(graph: &KnowledgeGraph) -> KnowledgeGraphDto {
    KnowledgeGraphDto {
        nodes: graph.nodes.iter().map(graph_node_to_dto).collect(),
        edges: graph.edges.iter().map(graph_edge_to_dto).collect(),
    }
}

fn graph_node_to_dto(node: &GraphNode) -> GraphNodeDto {
    GraphNodeDto {
        id: node.id.clone(),
        node_type: node.node_type.clone(),
        name: node.name.clone(),
    }
}

fn graph_edge_to_dto(edge: &GraphEdge) -> GraphEdgeDto {
    GraphEdgeDto {
        id: edge.id.clone(),
        source: edge.source.clone(),
        target: edge.target.clone(),
        edge_type: edge.edge_type.clone(),
    }
}

fn ai_context_to_dto(context: &ContextBuildResult) -> AiContextResultDto {
    AiContextResultDto {
        chunks: context
            .chunks
            .iter()
            .map(|chunk| AiContextChunkDto {
                id: chunk.id.clone(),
                path: chunk.path.clone(),
                chunk_index: chunk.chunk_index,
                start_line: chunk.start_line,
                end_line: chunk.end_line,
                content: chunk.content.clone(),
                char_count: chunk.char_count,
                token_estimate: chunk.token_estimate,
            })
            .collect(),
        skipped_files: context
            .skipped_files
            .iter()
            .map(|file| SkippedContextFileDto {
                path: file.path.clone(),
                reason: skipped_context_file_reason_to_string(&file.reason),
            })
            .collect(),
    }
}

fn ai_embeddings_to_dto(embeddings: &EmbeddingBuildResult) -> AiEmbeddingResultDto {
    AiEmbeddingResultDto {
        embeddings: embeddings
            .embeddings
            .iter()
            .map(|embedding| AiEmbeddingVectorDto {
                id: embedding.id.clone(),
                chunk_id: embedding.chunk_id.clone(),
                path: embedding.path.clone(),
                dimensions: embedding.dimensions,
                model: embedding.model.clone(),
                values: embedding.values.clone(),
            })
            .collect(),
        dimensions: embeddings.dimensions,
        model: embeddings.model.clone(),
    }
}

fn ai_vector_store_to_dto(vector_store: &VectorStoreBuildResult) -> AiVectorStoreResultDto {
    AiVectorStoreResultDto {
        embedding_count: vector_store.embedding_count,
        dimensions: vector_store.dimensions,
        model: vector_store.model.clone(),
    }
}

fn ai_vector_snapshot_summary_to_dto(snapshot: &AiVectorSnapshot) -> AiVectorSnapshotSummaryDto {
    AiVectorSnapshotSummaryDto {
        id: snapshot.id.0.clone(),
        repository_id: snapshot.repository_id.0.clone(),
        scan_id: snapshot.scan_id.as_ref().map(|scan_id| scan_id.0.clone()),
        embedding_count: snapshot.embedding_count,
        dimensions: snapshot.dimensions,
        model: snapshot.model.clone(),
        created_at_ms: snapshot.created_at_ms,
    }
}

fn ai_vector_snapshot_to_dto(snapshot: &AiVectorSnapshot) -> AiVectorSnapshotDto {
    AiVectorSnapshotDto {
        id: snapshot.id.0.clone(),
        repository_id: snapshot.repository_id.0.clone(),
        scan_id: snapshot.scan_id.as_ref().map(|scan_id| scan_id.0.clone()),
        embeddings: snapshot
            .embeddings
            .iter()
            .map(ai_vector_embedding_to_dto)
            .collect(),
        embedding_count: snapshot.embedding_count,
        dimensions: snapshot.dimensions,
        model: snapshot.model.clone(),
        created_at_ms: snapshot.created_at_ms,
    }
}

fn ai_vector_embedding_to_dto(embedding: &AiVectorEmbedding) -> AiEmbeddingVectorDto {
    AiEmbeddingVectorDto {
        id: embedding.id.clone(),
        chunk_id: embedding.chunk_id.clone(),
        path: embedding.path.clone(),
        dimensions: embedding.dimensions,
        model: embedding.model.clone(),
        values: embedding.values.clone(),
    }
}

fn ai_vector_snapshot_to_store(snapshot: &AiVectorSnapshot) -> VectorStoreBuildResult {
    VectorStoreBuildResult {
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
        embedding_count: snapshot.embedding_count,
        dimensions: snapshot.dimensions,
        model: snapshot.model.clone(),
    }
}

fn ai_retrieval_to_dto(result: &RetrievalResult) -> AiRetrievalResultDto {
    AiRetrievalResultDto {
        query: result.query.clone(),
        matches: result
            .matches
            .iter()
            .map(|match_item| AiRetrievalMatchDto {
                chunk_id: match_item.chunk_id.clone(),
                path: match_item.path.clone(),
                start_line: match_item.start_line,
                end_line: match_item.end_line,
                content: match_item.content.clone(),
                score: match_item.score,
                vector_score: match_item.vector_score,
                lexical_score: match_item.lexical_score,
            })
            .collect(),
    }
}

fn ai_context_bundle_to_dto(bundle: &ContextBundle) -> AiContextBundleDto {
    AiContextBundleDto {
        query: bundle.query.clone(),
        content: bundle.content.clone(),
        sources: bundle
            .sources
            .iter()
            .map(|source| AiContextBundleSourceDto {
                source_id: source.source_id.clone(),
                chunk_id: source.chunk_id.clone(),
                path: source.path.clone(),
                start_line: source.start_line,
                end_line: source.end_line,
                score: source.score,
                token_estimate: source.token_estimate,
            })
            .collect(),
        token_estimate: bundle.token_estimate,
        max_tokens: bundle.max_tokens,
        truncated: bundle.truncated,
    }
}

fn ai_chat_response_to_dto(response: &ChatResponse) -> AiChatResponseDto {
    AiChatResponseDto {
        question: response.question.clone(),
        answer: response.answer.clone(),
        citations: response
            .citations
            .iter()
            .map(|citation| AiChatCitationDto {
                source_id: citation.source_id.clone(),
                path: citation.path.clone(),
                start_line: citation.start_line,
                end_line: citation.end_line,
                score: citation.score,
            })
            .collect(),
        context: ai_context_bundle_to_dto(&response.context),
        model: response.model.clone(),
    }
}

fn chat_session_to_dto(session: &ChatSession) -> ChatSessionDto {
    ChatSessionDto {
        id: session.id.0.clone(),
        repository_id: session.repository_id.0.clone(),
        title: session.title.clone(),
        created_at_ms: session.created_at_ms,
        updated_at_ms: session.updated_at_ms,
    }
}

fn chat_message_to_dto(message: &ChatMessage) -> ChatMessageDto {
    ChatMessageDto {
        id: message.id.0.clone(),
        repository_id: message.repository_id.0.clone(),
        session_id: message.session_id.0.clone(),
        role: message.role.as_str().to_string(),
        content: message.content.clone(),
        model: message.model.clone(),
        citation_count: message.citation_count,
        created_at_ms: message.created_at_ms,
    }
}

fn repository_memory_to_dto(memory: &RepositoryMemory) -> RepositoryMemoryDto {
    RepositoryMemoryDto {
        repository_id: memory.repository_id.clone(),
        repository_name: memory.repository_name.clone(),
        path: memory.path.clone(),
        scan_id: memory.scan_id.clone(),
        files_count: memory.files_count,
        folders_count: memory.folders_count,
        technologies: memory
            .technologies
            .iter()
            .map(repository_memory_technology_to_dto)
            .collect(),
        graph_nodes: memory.graph_nodes,
        graph_edges: memory.graph_edges,
        document_count: memory.document_count,
        diagram_count: memory.diagram_count,
        last_export_path: memory.last_export_path.clone(),
        ai_context_chunks: memory.ai_context_chunks,
        ai_embedding_count: memory.ai_embedding_count,
        ai_model: memory.ai_model.clone(),
        recent_questions: memory.recent_questions.clone(),
        updated_at_ms: memory.updated_at_ms,
    }
}

fn cloud_readiness_to_dto(readiness: &CloudReadiness) -> CloudReadinessDto {
    CloudReadinessDto {
        enabled: readiness.enabled,
        status: readiness.status.clone(),
        support_level: readiness.support_level.clone(),
        tenant_types: readiness.tenant_types.clone(),
        sync_modes: readiness.sync_modes.clone(),
        deployment_models: readiness.deployment_models.clone(),
        network_required: readiness.network_required,
        cloud_tables_enabled: readiness.cloud_tables_enabled,
    }
}

fn mcp_readiness_to_dto(readiness: &McpReadiness) -> McpReadinessDto {
    McpReadinessDto {
        enabled: readiness.enabled,
        status: readiness.status.clone(),
        support_level: readiness.support_level.clone(),
        transports: readiness.transports.clone(),
        resources: readiness.resources.clone(),
        tools: readiness.tools.clone(),
        prompts: readiness.prompts.clone(),
        authentication_required: readiness.authentication_required,
        runtime_crate_enabled: readiness.runtime_crate_enabled,
    }
}

fn plugin_readiness_to_dto(readiness: &PluginReadiness) -> PluginReadinessDto {
    PluginReadinessDto {
        enabled: readiness.enabled,
        status: readiness.status.clone(),
        support_level: readiness.support_level.clone(),
        categories: readiness.categories.clone(),
        lifecycle: readiness.lifecycle.clone(),
        permissions: readiness.permissions.clone(),
        manifest_required_fields: readiness.manifest_required_fields.clone(),
        sandbox_required: readiness.sandbox_required,
        marketplace_enabled: readiness.marketplace_enabled,
        runtime_crate_enabled: readiness.runtime_crate_enabled,
        registry_table_enabled: readiness.registry_table_enabled,
        network_access_enabled: readiness.network_access_enabled,
    }
}

fn performance_readiness_to_dto(readiness: &PerformanceReadiness) -> PerformanceReadinessDto {
    PerformanceReadinessDto {
        enabled: readiness.enabled,
        status: readiness.status.clone(),
        support_level: readiness.support_level.clone(),
        analysis_domains: readiness.analysis_domains.clone(),
        metrics: readiness.metrics.clone(),
        risk_levels: readiness.risk_levels.clone(),
        thresholds: readiness.thresholds.clone(),
        report_types: readiness.report_types.clone(),
        runtime_crate_enabled: readiness.runtime_crate_enabled,
        report_tables_enabled: readiness.report_tables_enabled,
        profiling_enabled: readiness.profiling_enabled,
        recommendations_enabled: readiness.recommendations_enabled,
    }
}

fn security_readiness_to_dto(readiness: &SecurityReadiness) -> SecurityReadinessDto {
    SecurityReadinessDto {
        enabled: readiness.enabled,
        status: readiness.status.clone(),
        support_level: readiness.support_level.clone(),
        security_domains: readiness.security_domains.clone(),
        secret_categories: readiness.secret_categories.clone(),
        dependency_sources: readiness.dependency_sources.clone(),
        owasp_mappings: readiness.owasp_mappings.clone(),
        risk_levels: readiness.risk_levels.clone(),
        report_types: readiness.report_types.clone(),
        runtime_crate_enabled: readiness.runtime_crate_enabled,
        report_tables_enabled: readiness.report_tables_enabled,
        secret_value_export_enabled: readiness.secret_value_export_enabled,
        vulnerability_network_lookup_enabled: readiness.vulnerability_network_lookup_enabled,
        dashboard_enabled: readiness.dashboard_enabled,
    }
}

fn git_readiness_to_dto(readiness: &GitReadiness) -> GitReadinessDto {
    GitReadinessDto {
        enabled: readiness.enabled,
        status: readiness.status.clone(),
        support_level: readiness.support_level.clone(),
        data_sources: readiness.data_sources.clone(),
        analysis_domains: readiness.analysis_domains.clone(),
        graph_node_types: readiness.graph_node_types.clone(),
        relationship_types: readiness.relationship_types.clone(),
        report_types: readiness.report_types.clone(),
        runtime_crate_enabled: readiness.runtime_crate_enabled,
        history_mutation_enabled: readiness.history_mutation_enabled,
        repository_timeline_enabled: readiness.repository_timeline_enabled,
        ownership_analysis_enabled: readiness.ownership_analysis_enabled,
        hotspot_analysis_enabled: readiness.hotspot_analysis_enabled,
        drift_detection_enabled: readiness.drift_detection_enabled,
    }
}

fn backend_readiness_to_dto(readiness: &BackendReadiness) -> BackendReadinessDto {
    BackendReadinessDto {
        architecture: readiness.architecture.clone(),
        command_layer: readiness.command_layer.clone(),
        structured_errors: readiness.structured_errors,
        event_bus_enabled: readiness.event_bus_enabled,
        storage_enabled: readiness.storage_enabled,
        business_logic_in_rust: readiness.business_logic_in_rust,
        network_calls_enabled: readiness.network_calls_enabled,
        service_crates: readiness.service_crates.clone(),
        future_crates: readiness.future_crates.clone(),
    }
}

fn storage_readiness_to_dto(readiness: &StorageReadiness) -> StorageReadinessDto {
    StorageReadinessDto {
        primary_store: readiness.primary_store.clone(),
        operational_tables: readiness.operational_tables.clone(),
        snapshot_tables: readiness.snapshot_tables.clone(),
        future_tables: readiness.future_tables.clone(),
        vector_store_enabled: readiness.vector_store_enabled,
        vector_store: readiness.vector_store.clone(),
        search_index_enabled: readiness.search_index_enabled,
        search_index: readiness.search_index.clone(),
        graph_engine_enabled: readiness.graph_engine_enabled,
        graph_persistence: readiness.graph_persistence.clone(),
        json_snapshots_enabled: readiness.json_snapshots_enabled,
        cloud_tables_enabled: readiness.cloud_tables_enabled,
    }
}

fn repository_memory_technology_to_dto(
    technology: &RepositoryMemoryTechnology,
) -> RepositoryMemoryTechnologyDto {
    RepositoryMemoryTechnologyDto {
        category: technology.category.clone(),
        name: technology.name.clone(),
        version: technology.version.clone(),
    }
}

fn remember_ai_question(state: &tauri::State<'_, AppState>, question: String) -> CommandResult<()> {
    let mut questions = state.ai_questions.lock().map_err(lock_error)?;
    questions.push(question);
    if questions.len() > 10 {
        let overflow = questions.len() - 10;
        questions.drain(0..overflow);
    }
    Ok(())
}

fn skipped_context_file_reason_to_string(reason: &SkippedContextFileReason) -> String {
    reason.as_str().to_string()
}

fn category_to_string(category: &TechnologyCategory) -> String {
    category.as_str().to_string()
}

fn document_type_to_string(document_type: &DocumentType) -> String {
    match document_type {
        DocumentType::Readme => "README",
        DocumentType::Architecture => "Architecture",
        DocumentType::Modules => "Modules",
        DocumentType::ApiSummary => "API Summary",
        DocumentType::DatabaseSummary => "Database Summary",
        DocumentType::Onboarding => "Onboarding",
        DocumentType::AiContext => "AI Context",
    }
    .to_string()
}

fn diagram_type_to_string(diagram_type: &DiagramType) -> String {
    match diagram_type {
        DiagramType::Class => "Class",
        DiagramType::Component => "Component",
        DiagramType::Dependency => "Dependency",
        DiagramType::Erd => "ERD",
        DiagramType::FolderStructure => "Folder Structure",
        DiagramType::Package => "Package",
        DiagramType::ArchitectureOverview => "Architecture Overview",
    }
    .to_string()
}

fn diagram_format_to_string(format: &DiagramFormat) -> String {
    match format {
        DiagramFormat::PlantUml => "PlantUML",
        DiagramFormat::Svg => "SVG",
    }
    .to_string()
}

fn lock_error<T>(error: std::sync::PoisonError<T>) -> CommandError {
    command_error(
        "application.state_lock_failed",
        format!("Application state lock failed: {error}"),
    )
}

fn command_error(code: &str, message: impl Into<String>) -> CommandError {
    CommandError::new(code, message)
}

fn publish_event(state: &tauri::State<'_, AppState>, event: DomainEvent) {
    state.event_bus.publish(event);
}

#[cfg(test)]
mod tests {
    use super::CommandError;
    use devatlas_common::DevAtlasError;

    #[test]
    fn converts_domain_errors_to_structured_command_errors() {
        let error = CommandError::from(DevAtlasError::new(
            "repository.path_missing",
            "Repository path does not exist.",
        ));

        assert_eq!(error.code, "repository.path_missing");
        assert_eq!(error.message, "Repository path does not exist.");
    }
}
