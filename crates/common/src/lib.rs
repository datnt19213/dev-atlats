use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DOMAIN_EVENT_VERSION: &str = "1.0";
pub const DOMAIN_EVENT_TYPES: &[&str] = &[
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
];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepositoryId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScanId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagramId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphSnapshotId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AiVectorSnapshotId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChatSessionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChatMessageId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    pub id: RepositoryId,
    pub name: String,
    pub path: RepositoryPath,
    pub created_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryPath(PathBuf);

impl RepositoryPath {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, DevAtlasError> {
        let path = path.into();
        if !path.exists() {
            return Err(DevAtlasError::new(
                "repository.path_missing",
                format!("Repository path does not exist: {}", path.display()),
            ));
        }
        if !path.is_dir() {
            return Err(DevAtlasError::new(
                "repository.path_not_directory",
                format!("Repository path is not a directory: {}", path.display()),
            ));
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Technology {
    pub category: TechnologyCategory,
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TechnologyCategory {
    Language,
    Framework,
    Database,
    Orm,
    PackageManager,
    Infrastructure,
    Library,
}

impl TechnologyCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Language => "Language",
            Self::Framework => "Framework",
            Self::Database => "Database",
            Self::Orm => "ORM",
            Self::PackageManager => "Package Manager",
            Self::Infrastructure => "Infrastructure",
            Self::Library => "Library",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryFile {
    pub path: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub scan_id: ScanId,
    pub repository_id: RepositoryId,
    pub status: ScanStatus,
    pub files_count: usize,
    pub folders_count: usize,
    pub technologies: Vec<Technology>,
    pub files: Vec<RepositoryFile>,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedDocument {
    pub id: DocumentId,
    pub path: String,
    pub document_type: DocumentType,
    pub content: String,
    pub semantic_plan: DocumentationPlan,
    pub quality: DocumentationQuality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationPlan {
    pub audience: String,
    pub intent: String,
    pub sections: Vec<DocumentationSectionPlan>,
    pub evidence_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationSectionPlan {
    pub title: String,
    pub purpose: String,
    pub evidence_type: String,
    pub required_signals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationQuality {
    pub coverage_score: u8,
    pub semantic_score: u8,
    pub source_count: usize,
    pub symbol_count: usize,
    pub graph_edge_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentType {
    Readme,
    Architecture,
    Modules,
    ApiSummary,
    DatabaseSummary,
    Onboarding,
    AiContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramResult {
    pub id: DiagramId,
    pub path: String,
    pub diagram_type: DiagramType,
    pub format: DiagramFormat,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramType {
    Class,
    Component,
    Dependency,
    Erd,
    FolderStructure,
    Package,
    ArchitectureOverview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramFormat {
    Mermaid,
    PlantUml,
    Svg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPackage {
    pub id: ExportId,
    pub path: String,
    pub artifacts_dir: String,
    pub artifact_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryMemoryTechnology {
    pub category: String,
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryMemory {
    pub repository_id: String,
    pub repository_name: String,
    pub path: String,
    pub scan_id: Option<String>,
    pub files_count: usize,
    pub folders_count: usize,
    pub technologies: Vec<RepositoryMemoryTechnology>,
    pub graph_nodes: usize,
    pub graph_edges: usize,
    pub document_count: usize,
    pub diagram_count: usize,
    pub last_export_path: Option<String>,
    pub ai_context_chunks: usize,
    pub ai_embedding_count: usize,
    pub ai_model: Option<String>,
    pub recent_questions: Vec<String>,
    pub updated_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudReadiness {
    pub enabled: bool,
    pub status: String,
    pub support_level: String,
    pub tenant_types: Vec<String>,
    pub sync_modes: Vec<String>,
    pub deployment_models: Vec<String>,
    pub network_required: bool,
    pub cloud_tables_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpReadiness {
    pub enabled: bool,
    pub status: String,
    pub support_level: String,
    pub transports: Vec<String>,
    pub resources: Vec<String>,
    pub tools: Vec<String>,
    pub prompts: Vec<String>,
    pub authentication_required: bool,
    pub runtime_crate_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginReadiness {
    pub enabled: bool,
    pub status: String,
    pub support_level: String,
    pub categories: Vec<String>,
    pub lifecycle: Vec<String>,
    pub permissions: Vec<String>,
    pub manifest_required_fields: Vec<String>,
    pub sandbox_required: bool,
    pub marketplace_enabled: bool,
    pub runtime_crate_enabled: bool,
    pub registry_table_enabled: bool,
    pub network_access_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformanceReadiness {
    pub enabled: bool,
    pub status: String,
    pub support_level: String,
    pub analysis_domains: Vec<String>,
    pub metrics: Vec<String>,
    pub risk_levels: Vec<String>,
    pub thresholds: Vec<String>,
    pub report_types: Vec<String>,
    pub runtime_crate_enabled: bool,
    pub report_tables_enabled: bool,
    pub profiling_enabled: bool,
    pub recommendations_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityReadiness {
    pub enabled: bool,
    pub status: String,
    pub support_level: String,
    pub security_domains: Vec<String>,
    pub secret_categories: Vec<String>,
    pub dependency_sources: Vec<String>,
    pub owasp_mappings: Vec<String>,
    pub risk_levels: Vec<String>,
    pub report_types: Vec<String>,
    pub runtime_crate_enabled: bool,
    pub report_tables_enabled: bool,
    pub secret_value_export_enabled: bool,
    pub vulnerability_network_lookup_enabled: bool,
    pub dashboard_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitReadiness {
    pub enabled: bool,
    pub status: String,
    pub support_level: String,
    pub data_sources: Vec<String>,
    pub analysis_domains: Vec<String>,
    pub graph_node_types: Vec<String>,
    pub relationship_types: Vec<String>,
    pub report_types: Vec<String>,
    pub runtime_crate_enabled: bool,
    pub history_mutation_enabled: bool,
    pub repository_timeline_enabled: bool,
    pub ownership_analysis_enabled: bool,
    pub hotspot_analysis_enabled: bool,
    pub drift_detection_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendReadiness {
    pub architecture: String,
    pub command_layer: String,
    pub structured_errors: bool,
    pub event_bus_enabled: bool,
    pub storage_enabled: bool,
    pub business_logic_in_rust: bool,
    pub network_calls_enabled: bool,
    pub service_crates: Vec<String>,
    pub future_crates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageReadiness {
    pub primary_store: String,
    pub operational_tables: Vec<String>,
    pub snapshot_tables: Vec<String>,
    pub future_tables: Vec<String>,
    pub vector_store_enabled: bool,
    pub vector_store: String,
    pub search_index_enabled: bool,
    pub search_index: String,
    pub graph_engine_enabled: bool,
    pub graph_persistence: String,
    pub json_snapshots_enabled: bool,
    pub cloud_tables_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphSnapshot {
    pub id: GraphSnapshotId,
    pub repository_id: RepositoryId,
    pub scan_id: Option<ScanId>,
    pub graph: KnowledgeGraph,
    pub node_count: usize,
    pub edge_count: usize,
    pub created_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiVectorEmbedding {
    pub id: String,
    pub chunk_id: String,
    pub path: String,
    pub dimensions: usize,
    pub model: String,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiVectorSnapshot {
    pub id: AiVectorSnapshotId,
    pub repository_id: RepositoryId,
    pub scan_id: Option<ScanId>,
    pub embeddings: Vec<AiVectorEmbedding>,
    pub embedding_count: usize,
    pub dimensions: usize,
    pub model: String,
    pub created_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSession {
    pub id: ChatSessionId,
    pub repository_id: RepositoryId,
    pub title: String,
    pub created_at_ms: u128,
    pub updated_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatMessageRole {
    User,
    Assistant,
    System,
}

impl ChatMessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Assistant => "Assistant",
            Self::System => "System",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: ChatMessageId,
    pub repository_id: RepositoryId,
    pub session_id: ChatSessionId,
    pub role: ChatMessageRole,
    pub content: String,
    pub model: Option<String>,
    pub citation_count: usize,
    pub created_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainEvent {
    pub event_id: String,
    pub correlation_id: String,
    pub event_type: String,
    pub version: String,
    pub timestamp_ms: u128,
    pub payload: DomainEventPayload,
}

impl DomainEvent {
    pub fn new(correlation_id: impl Into<String>, payload: DomainEventPayload) -> Self {
        let timestamp_ms = now_ms();
        let event_type = payload.event_type().to_string();
        Self {
            event_id: stable_id("event", &format!("{event_type}-{timestamp_ms}")),
            correlation_id: correlation_id.into(),
            event_type,
            version: DOMAIN_EVENT_VERSION.to_string(),
            timestamp_ms,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all_fields = "camelCase")]
pub enum DomainEventPayload {
    RepositoryOpened {
        repository_id: String,
        path: String,
    },
    ScanStarted {
        scan_id: String,
        repository_id: String,
    },
    ScanCompleted {
        scan_id: String,
        repository_id: String,
        duration_ms: u128,
    },
    GraphBuilt {
        repository_id: String,
        node_count: usize,
        edge_count: usize,
    },
    DocumentationGenerated {
        repository_id: String,
        document_count: usize,
    },
    DiagramGenerated {
        repository_id: String,
        diagram_count: usize,
    },
    ExportCompleted {
        repository_id: String,
        path: String,
    },
    AiContextBuilt {
        repository_id: String,
        chunk_count: usize,
        skipped_file_count: usize,
    },
    AiEmbeddingsBuilt {
        repository_id: String,
        embedding_count: usize,
        dimensions: usize,
        model: String,
    },
    AiVectorStoreBuilt {
        repository_id: String,
        embedding_count: usize,
        dimensions: usize,
        model: String,
    },
    AiRetrievalCompleted {
        repository_id: String,
        query: String,
        match_count: usize,
    },
    AiPersistedRetrievalCompleted {
        repository_id: String,
        snapshot_id: String,
        query: String,
        match_count: usize,
    },
    AiContextBundleBuilt {
        repository_id: String,
        query: String,
        source_count: usize,
        token_estimate: usize,
        truncated: bool,
    },
    AiPersistedContextBundleBuilt {
        repository_id: String,
        snapshot_id: String,
        query: String,
        source_count: usize,
        token_estimate: usize,
        truncated: bool,
    },
    AiChatResponded {
        repository_id: String,
        question: String,
        citation_count: usize,
        model: String,
    },
    AiPersistedChatResponded {
        repository_id: String,
        snapshot_id: String,
        question: String,
        citation_count: usize,
        model: String,
    },
    AiChatSessionStarted {
        repository_id: String,
        session_id: String,
        title: String,
    },
    AiChatMessageSaved {
        repository_id: String,
        session_id: String,
        message_id: String,
        role: String,
    },
    RepositoryMemoryBuilt {
        repository_id: String,
        files_count: usize,
        graph_nodes: usize,
        ai_context_chunks: usize,
    },
    GraphSnapshotSaved {
        repository_id: String,
        snapshot_id: String,
        node_count: usize,
        edge_count: usize,
    },
    AiVectorSnapshotSaved {
        repository_id: String,
        snapshot_id: String,
        embedding_count: usize,
        dimensions: usize,
        model: String,
    },
}

impl DomainEventPayload {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::RepositoryOpened { .. } => "Repository.RepositoryOpened",
            Self::ScanStarted { .. } => "Scanner.ScanStarted",
            Self::ScanCompleted { .. } => "Scanner.ScanCompleted",
            Self::GraphBuilt { .. } => "Graph.GraphBuilt",
            Self::DocumentationGenerated { .. } => "Documentation.DocumentationGenerated",
            Self::DiagramGenerated { .. } => "Diagram.DiagramGenerated",
            Self::ExportCompleted { .. } => "Export.ExportCompleted",
            Self::AiContextBuilt { .. } => "AI.ContextBuilt",
            Self::AiEmbeddingsBuilt { .. } => "AI.EmbeddingsBuilt",
            Self::AiVectorStoreBuilt { .. } => "AI.VectorStoreBuilt",
            Self::AiRetrievalCompleted { .. } => "AI.RetrievalCompleted",
            Self::AiPersistedRetrievalCompleted { .. } => "AI.PersistedRetrievalCompleted",
            Self::AiContextBundleBuilt { .. } => "AI.ContextBundleBuilt",
            Self::AiPersistedContextBundleBuilt { .. } => "AI.PersistedContextBundleBuilt",
            Self::AiChatResponded { .. } => "AI.ChatResponded",
            Self::AiPersistedChatResponded { .. } => "AI.PersistedChatResponded",
            Self::AiChatSessionStarted { .. } => "AI.ChatSessionStarted",
            Self::AiChatMessageSaved { .. } => "AI.ChatMessageSaved",
            Self::RepositoryMemoryBuilt { .. } => "Repository.MemoryBuilt",
            Self::GraphSnapshotSaved { .. } => "Graph.SnapshotSaved",
            Self::AiVectorSnapshotSaved { .. } => "AI.VectorSnapshotSaved",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevAtlasError {
    pub code: String,
    pub message: String,
}

impl DevAtlasError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl Display for DevAtlasError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DevAtlasError {}

pub type DevAtlasResult<T> = Result<T, DevAtlasError>;

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub fn stable_id(prefix: &str, input: &str) -> String {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("{prefix}-{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::{stable_id, DomainEvent, DomainEventPayload, DOMAIN_EVENT_TYPES};
    use std::collections::HashSet;

    #[test]
    fn stable_id_is_deterministic() {
        assert_eq!(stable_id("repo", "sample"), stable_id("repo", "sample"));
    }

    #[test]
    fn domain_event_uses_versioned_type() {
        let event = DomainEvent::new(
            "repo-1",
            DomainEventPayload::RepositoryOpened {
                repository_id: "repo-1".to_string(),
                path: "sample".to_string(),
            },
        );
        assert_eq!(event.event_type, "Repository.RepositoryOpened");
        assert_eq!(event.version, "1.0");
    }

    #[test]
    fn mvp_event_types_are_unique_and_serializable() {
        let payloads = vec![
            DomainEventPayload::RepositoryOpened {
                repository_id: "repo-1".to_string(),
                path: "sample".to_string(),
            },
            DomainEventPayload::ScanStarted {
                scan_id: "scan-1".to_string(),
                repository_id: "repo-1".to_string(),
            },
            DomainEventPayload::ScanCompleted {
                scan_id: "scan-1".to_string(),
                repository_id: "repo-1".to_string(),
                duration_ms: 42,
            },
            DomainEventPayload::GraphBuilt {
                repository_id: "repo-1".to_string(),
                node_count: 10,
                edge_count: 12,
            },
            DomainEventPayload::DocumentationGenerated {
                repository_id: "repo-1".to_string(),
                document_count: 5,
            },
            DomainEventPayload::DiagramGenerated {
                repository_id: "repo-1".to_string(),
                diagram_count: 3,
            },
            DomainEventPayload::ExportCompleted {
                repository_id: "repo-1".to_string(),
                path: "project-knowledge.zip".to_string(),
            },
            DomainEventPayload::AiContextBuilt {
                repository_id: "repo-1".to_string(),
                chunk_count: 8,
                skipped_file_count: 2,
            },
            DomainEventPayload::AiEmbeddingsBuilt {
                repository_id: "repo-1".to_string(),
                embedding_count: 8,
                dimensions: 128,
                model: "devatlas-local-hash-v1".to_string(),
            },
            DomainEventPayload::AiVectorStoreBuilt {
                repository_id: "repo-1".to_string(),
                embedding_count: 8,
                dimensions: 128,
                model: "devatlas-local-hash-v1".to_string(),
            },
            DomainEventPayload::AiRetrievalCompleted {
                repository_id: "repo-1".to_string(),
                query: "scan repository".to_string(),
                match_count: 3,
            },
            DomainEventPayload::AiPersistedRetrievalCompleted {
                repository_id: "repo-1".to_string(),
                snapshot_id: "ai-vector-snapshot-1".to_string(),
                query: "scan repository".to_string(),
                match_count: 3,
            },
            DomainEventPayload::AiContextBundleBuilt {
                repository_id: "repo-1".to_string(),
                query: "scan repository".to_string(),
                source_count: 3,
                token_estimate: 512,
                truncated: false,
            },
            DomainEventPayload::AiPersistedContextBundleBuilt {
                repository_id: "repo-1".to_string(),
                snapshot_id: "ai-vector-snapshot-1".to_string(),
                query: "scan repository".to_string(),
                source_count: 3,
                token_estimate: 512,
                truncated: false,
            },
            DomainEventPayload::AiChatResponded {
                repository_id: "repo-1".to_string(),
                question: "How does scanning work?".to_string(),
                citation_count: 3,
                model: "devatlas-local-grounded-v1".to_string(),
            },
            DomainEventPayload::AiPersistedChatResponded {
                repository_id: "repo-1".to_string(),
                snapshot_id: "ai-vector-snapshot-1".to_string(),
                question: "How does scanning work?".to_string(),
                citation_count: 3,
                model: "devatlas-local-grounded-v1".to_string(),
            },
            DomainEventPayload::AiChatSessionStarted {
                repository_id: "repo-1".to_string(),
                session_id: "chat-session-1".to_string(),
                title: "How does scanning work?".to_string(),
            },
            DomainEventPayload::AiChatMessageSaved {
                repository_id: "repo-1".to_string(),
                session_id: "chat-session-1".to_string(),
                message_id: "chat-message-1".to_string(),
                role: "User".to_string(),
            },
            DomainEventPayload::RepositoryMemoryBuilt {
                repository_id: "repo-1".to_string(),
                files_count: 10,
                graph_nodes: 7,
                ai_context_chunks: 4,
            },
            DomainEventPayload::GraphSnapshotSaved {
                repository_id: "repo-1".to_string(),
                snapshot_id: "graph-snapshot-1".to_string(),
                node_count: 7,
                edge_count: 9,
            },
            DomainEventPayload::AiVectorSnapshotSaved {
                repository_id: "repo-1".to_string(),
                snapshot_id: "ai-vector-snapshot-1".to_string(),
                embedding_count: 8,
                dimensions: 128,
                model: "devatlas-local-hash-v1".to_string(),
            },
        ];
        let event_types = payloads
            .iter()
            .map(DomainEventPayload::event_type)
            .collect::<HashSet<&str>>();
        let contract_types = DOMAIN_EVENT_TYPES
            .iter()
            .copied()
            .collect::<HashSet<&str>>();

        assert_eq!(event_types.len(), payloads.len());
        assert_eq!(event_types, contract_types);

        for payload in payloads {
            let event = DomainEvent::new("repo-1", payload);
            let json = serde_json::to_value(&event).expect("event should serialize");

            assert_eq!(json["correlationId"], "repo-1");
            assert_eq!(json["version"], "1.0");
            assert!(json["eventId"].as_str().is_some());
            assert!(json["timestampMs"].as_u64().is_some());
            assert!(json["payload"]["type"].as_str().is_some());
        }

        let repository_opened = DomainEvent::new(
            "repo-1",
            DomainEventPayload::RepositoryOpened {
                repository_id: "repo-1".to_string(),
                path: "sample".to_string(),
            },
        );
        let json = serde_json::to_value(repository_opened).expect("event should serialize");
        assert_eq!(json["payload"]["data"]["repositoryId"], "repo-1");
    }
}
