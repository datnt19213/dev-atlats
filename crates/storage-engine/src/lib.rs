use devatlas_common::{
    stable_id, AiVectorEmbedding, AiVectorSnapshot, AiVectorSnapshotId, ChatMessage, ChatMessageId,
    ChatMessageRole, ChatSession, ChatSessionId, DevAtlasError, DevAtlasResult, ExportPackage,
    GraphSnapshot, GraphSnapshotId, KnowledgeGraph, Repository, RepositoryId, ScanId, ScanResult,
    ScanStatus, Technology, TechnologyCategory,
};
use rusqlite::{params, Connection};
use std::path::Path;

pub const MVP_SQLITE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS repositories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    repository_type TEXT,
    created_at DATETIME,
    updated_at DATETIME
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    framework TEXT,
    language TEXT,
    FOREIGN KEY(repository_id) REFERENCES repositories(id)
);

CREATE TABLE IF NOT EXISTS scans (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    status TEXT NOT NULL,
    files_count INTEGER,
    folders_count INTEGER,
    duration_ms INTEGER,
    started_at DATETIME,
    completed_at DATETIME
);

CREATE TABLE IF NOT EXISTS technologies (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    category TEXT,
    name TEXT,
    version TEXT
);

CREATE TABLE IF NOT EXISTS exports (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    export_type TEXT,
    output_path TEXT,
    created_at DATETIME
);

CREATE TABLE IF NOT EXISTS graph_snapshots (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    scan_id TEXT,
    node_count INTEGER NOT NULL,
    edge_count INTEGER NOT NULL,
    graph_json TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    created_at DATETIME,
    FOREIGN KEY(repository_id) REFERENCES repositories(id)
);

CREATE TABLE IF NOT EXISTS ai_vector_snapshots (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    scan_id TEXT,
    embedding_count INTEGER NOT NULL,
    dimensions INTEGER NOT NULL,
    model TEXT NOT NULL,
    embeddings_json TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    created_at DATETIME,
    FOREIGN KEY(repository_id) REFERENCES repositories(id)
);

CREATE TABLE IF NOT EXISTS chat_sessions (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    title TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at DATETIME,
    updated_at DATETIME,
    FOREIGN KEY(repository_id) REFERENCES repositories(id)
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    model TEXT,
    citation_count INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    created_at DATETIME,
    FOREIGN KEY(repository_id) REFERENCES repositories(id),
    FOREIGN KEY(session_id) REFERENCES chat_sessions(id)
);
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePlan {
    pub schema: &'static str,
    pub tables: Vec<&'static str>,
}

pub struct StorageService;

impl StorageService {
    pub fn mvp_plan() -> StoragePlan {
        StoragePlan {
            schema: MVP_SQLITE_SCHEMA,
            tables: vec![
                "repositories",
                "projects",
                "scans",
                "technologies",
                "exports",
                "graph_snapshots",
                "ai_vector_snapshots",
                "chat_sessions",
                "chat_messages",
            ],
        }
    }
}

pub struct SqliteStorage {
    connection: Connection,
}

impl SqliteStorage {
    pub fn open(path: &Path) -> DevAtlasResult<Self> {
        let connection = Connection::open(path).map_err(sql_error)?;
        let storage = Self { connection };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn open_in_memory() -> DevAtlasResult<Self> {
        let connection = Connection::open_in_memory().map_err(sql_error)?;
        let storage = Self { connection };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn initialize(&self) -> DevAtlasResult<()> {
        self.connection
            .execute_batch(MVP_SQLITE_SCHEMA)
            .map_err(sql_error)?;
        Ok(())
    }

    pub fn save_repository(&self, repository: &Repository) -> DevAtlasResult<()> {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO repositories (id, name, path, repository_type, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                params![
                    repository.id.0,
                    repository.name,
                    repository.path.as_path().to_string_lossy().to_string(),
                    "local"
                ],
            )
            .map_err(sql_error)?;
        Ok(())
    }

    pub fn list_repositories(&self) -> DevAtlasResult<Vec<StoredRepository>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name, path FROM repositories ORDER BY updated_at DESC")
            .map_err(sql_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(StoredRepository {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                })
            })
            .map_err(sql_error)?;
        collect_rows(rows)
    }

    pub fn save_scan(&self, scan: &ScanResult) -> DevAtlasResult<()> {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO scans (id, repository_id, status, files_count, folders_count, duration_ms, started_at, completed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
                params![
                    scan.scan_id.0,
                    scan.repository_id.0,
                    scan_status_to_str(&scan.status),
                    scan.files_count as i64,
                    scan.folders_count as i64,
                    scan.duration_ms as i64,
                ],
            )
            .map_err(sql_error)?;
        self.save_technologies(&scan.repository_id.0, &scan.technologies)?;
        Ok(())
    }

    pub fn save_technologies(
        &self,
        repository_id: &str,
        technologies: &[Technology],
    ) -> DevAtlasResult<()> {
        self.connection
            .execute(
                "DELETE FROM technologies WHERE repository_id = ?1",
                params![repository_id],
            )
            .map_err(sql_error)?;
        for technology in technologies {
            self.connection
                .execute(
                    "INSERT OR REPLACE INTO technologies (id, repository_id, category, name, version)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        stable_id(
                            "technology",
                            &format!(
                                "{}-{}-{}",
                                repository_id,
                                technology.category.as_str(),
                                technology.name
                            )
                        ),
                        repository_id,
                        technology.category.as_str(),
                        technology.name,
                        technology.version,
                    ],
                )
                .map_err(sql_error)?;
        }
        Ok(())
    }

    pub fn list_technologies(&self, repository_id: &str) -> DevAtlasResult<Vec<Technology>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT category, name, version FROM technologies WHERE repository_id = ?1 ORDER BY category, name",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map(params![repository_id], |row| {
                let category: String = row.get(0)?;
                Ok(Technology {
                    category: technology_category_from_str(&category),
                    name: row.get(1)?,
                    version: row.get(2)?,
                })
            })
            .map_err(sql_error)?;
        collect_rows(rows)
    }

    pub fn save_export(&self, repository_id: &str, package: &ExportPackage) -> DevAtlasResult<()> {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO exports (id, repository_id, export_type, output_path, created_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'))",
                params![package.id.0, repository_id, "knowledge-package", package.path],
            )
            .map_err(sql_error)?;
        Ok(())
    }

    pub fn save_graph_snapshot(&self, snapshot: &GraphSnapshot) -> DevAtlasResult<()> {
        let graph_json = serde_json::to_string(&snapshot.graph).map_err(json_error)?;
        self.connection
            .execute(
                "INSERT OR REPLACE INTO graph_snapshots (id, repository_id, scan_id, node_count, edge_count, graph_json, created_at_ms, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
                params![
                    snapshot.id.0,
                    snapshot.repository_id.0,
                    snapshot.scan_id.as_ref().map(|scan_id| scan_id.0.clone()),
                    snapshot.node_count as i64,
                    snapshot.edge_count as i64,
                    graph_json,
                    snapshot.created_at_ms as i64,
                ],
            )
            .map_err(sql_error)?;
        Ok(())
    }

    pub fn list_graph_snapshots(
        &self,
        repository_id: &str,
    ) -> DevAtlasResult<Vec<StoredGraphSnapshot>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, repository_id, scan_id, node_count, edge_count, created_at_ms
                 FROM graph_snapshots
                 WHERE repository_id = ?1
                 ORDER BY created_at_ms DESC",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map(params![repository_id], |row| {
                Ok(StoredGraphSnapshot {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    scan_id: row.get(2)?,
                    node_count: row.get::<_, i64>(3)? as usize,
                    edge_count: row.get::<_, i64>(4)? as usize,
                    created_at_ms: row.get::<_, i64>(5)? as u128,
                })
            })
            .map_err(sql_error)?;
        collect_rows(rows)
    }

    pub fn load_graph_snapshot(&self, snapshot_id: &str) -> DevAtlasResult<GraphSnapshot> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, repository_id, scan_id, node_count, edge_count, graph_json, created_at_ms
                 FROM graph_snapshots
                 WHERE id = ?1",
            )
            .map_err(sql_error)?;
        statement
            .query_row(params![snapshot_id], |row| {
                let graph_json: String = row.get(5)?;
                let graph = serde_json::from_str::<KnowledgeGraph>(&graph_json)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                let scan_id = row.get::<_, Option<String>>(2)?.map(ScanId);
                Ok(GraphSnapshot {
                    id: GraphSnapshotId(row.get(0)?),
                    repository_id: RepositoryId(row.get(1)?),
                    scan_id,
                    node_count: row.get::<_, i64>(3)? as usize,
                    edge_count: row.get::<_, i64>(4)? as usize,
                    graph,
                    created_at_ms: row.get::<_, i64>(6)? as u128,
                })
            })
            .map_err(sql_error)
    }

    pub fn save_ai_vector_snapshot(&self, snapshot: &AiVectorSnapshot) -> DevAtlasResult<()> {
        let embeddings_json = serde_json::to_string(&snapshot.embeddings).map_err(json_error)?;
        self.connection
            .execute(
                "INSERT OR REPLACE INTO ai_vector_snapshots (id, repository_id, scan_id, embedding_count, dimensions, model, embeddings_json, created_at_ms, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
                params![
                    snapshot.id.0,
                    snapshot.repository_id.0,
                    snapshot.scan_id.as_ref().map(|scan_id| scan_id.0.clone()),
                    snapshot.embedding_count as i64,
                    snapshot.dimensions as i64,
                    snapshot.model,
                    embeddings_json,
                    snapshot.created_at_ms as i64,
                ],
            )
            .map_err(sql_error)?;
        Ok(())
    }

    pub fn list_ai_vector_snapshots(
        &self,
        repository_id: &str,
    ) -> DevAtlasResult<Vec<StoredAiVectorSnapshot>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, repository_id, scan_id, embedding_count, dimensions, model, created_at_ms
                 FROM ai_vector_snapshots
                 WHERE repository_id = ?1
                 ORDER BY created_at_ms DESC",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map(params![repository_id], |row| {
                Ok(StoredAiVectorSnapshot {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    scan_id: row.get(2)?,
                    embedding_count: row.get::<_, i64>(3)? as usize,
                    dimensions: row.get::<_, i64>(4)? as usize,
                    model: row.get(5)?,
                    created_at_ms: row.get::<_, i64>(6)? as u128,
                })
            })
            .map_err(sql_error)?;
        collect_rows(rows)
    }

    pub fn load_ai_vector_snapshot(&self, snapshot_id: &str) -> DevAtlasResult<AiVectorSnapshot> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, repository_id, scan_id, embedding_count, dimensions, model, embeddings_json, created_at_ms
                 FROM ai_vector_snapshots
                 WHERE id = ?1",
            )
            .map_err(sql_error)?;
        statement
            .query_row(params![snapshot_id], |row| {
                let embeddings_json: String = row.get(6)?;
                let embeddings = serde_json::from_str::<Vec<AiVectorEmbedding>>(&embeddings_json)
                    .map_err(|error| {
                    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
                })?;
                let scan_id = row.get::<_, Option<String>>(2)?.map(ScanId);
                Ok(AiVectorSnapshot {
                    id: AiVectorSnapshotId(row.get(0)?),
                    repository_id: RepositoryId(row.get(1)?),
                    scan_id,
                    embedding_count: row.get::<_, i64>(3)? as usize,
                    dimensions: row.get::<_, i64>(4)? as usize,
                    model: row.get(5)?,
                    embeddings,
                    created_at_ms: row.get::<_, i64>(7)? as u128,
                })
            })
            .map_err(sql_error)
    }

    pub fn save_chat_session(&self, session: &ChatSession) -> DevAtlasResult<()> {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO chat_sessions (id, repository_id, title, created_at_ms, updated_at_ms, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
                params![
                    session.id.0,
                    session.repository_id.0,
                    session.title,
                    session.created_at_ms as i64,
                    session.updated_at_ms as i64,
                ],
            )
            .map_err(sql_error)?;
        Ok(())
    }

    pub fn list_chat_sessions(&self, repository_id: &str) -> DevAtlasResult<Vec<ChatSession>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, repository_id, title, created_at_ms, updated_at_ms
                 FROM chat_sessions
                 WHERE repository_id = ?1
                 ORDER BY updated_at_ms DESC",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map(params![repository_id], |row| {
                Ok(ChatSession {
                    id: ChatSessionId(row.get(0)?),
                    repository_id: RepositoryId(row.get(1)?),
                    title: row.get(2)?,
                    created_at_ms: row.get::<_, i64>(3)? as u128,
                    updated_at_ms: row.get::<_, i64>(4)? as u128,
                })
            })
            .map_err(sql_error)?;
        collect_rows(rows)
    }

    pub fn load_chat_session(&self, session_id: &str) -> DevAtlasResult<ChatSession> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, repository_id, title, created_at_ms, updated_at_ms
                 FROM chat_sessions
                 WHERE id = ?1",
            )
            .map_err(sql_error)?;
        statement
            .query_row(params![session_id], |row| {
                Ok(ChatSession {
                    id: ChatSessionId(row.get(0)?),
                    repository_id: RepositoryId(row.get(1)?),
                    title: row.get(2)?,
                    created_at_ms: row.get::<_, i64>(3)? as u128,
                    updated_at_ms: row.get::<_, i64>(4)? as u128,
                })
            })
            .map_err(sql_error)
    }

    pub fn save_chat_message(&self, message: &ChatMessage) -> DevAtlasResult<()> {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO chat_messages (id, repository_id, session_id, role, content, model, citation_count, created_at_ms, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
                params![
                    message.id.0,
                    message.repository_id.0,
                    message.session_id.0,
                    message.role.as_str(),
                    message.content,
                    message.model,
                    message.citation_count as i64,
                    message.created_at_ms as i64,
                ],
            )
            .map_err(sql_error)?;
        self.connection
            .execute(
                "UPDATE chat_sessions SET updated_at_ms = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![message.created_at_ms as i64, message.session_id.0],
            )
            .map_err(sql_error)?;
        Ok(())
    }

    pub fn list_chat_messages(&self, session_id: &str) -> DevAtlasResult<Vec<ChatMessage>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, repository_id, session_id, role, content, model, citation_count, created_at_ms
                 FROM chat_messages
                 WHERE session_id = ?1
                 ORDER BY created_at_ms ASC",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map(params![session_id], |row| {
                let role: String = row.get(3)?;
                Ok(ChatMessage {
                    id: ChatMessageId(row.get(0)?),
                    repository_id: RepositoryId(row.get(1)?),
                    session_id: ChatSessionId(row.get(2)?),
                    role: chat_message_role_from_str(&role),
                    content: row.get(4)?,
                    model: row.get(5)?,
                    citation_count: row.get::<_, i64>(6)? as usize,
                    created_at_ms: row.get::<_, i64>(7)? as u128,
                })
            })
            .map_err(sql_error)?;
        collect_rows(rows)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredRepository {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredGraphSnapshot {
    pub id: String,
    pub repository_id: String,
    pub scan_id: Option<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub created_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAiVectorSnapshot {
    pub id: String,
    pub repository_id: String,
    pub scan_id: Option<String>,
    pub embedding_count: usize,
    pub dimensions: usize,
    pub model: String,
    pub created_at_ms: u128,
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> DevAtlasResult<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row.map_err(sql_error)?);
    }
    Ok(values)
}

fn scan_status_to_str(status: &ScanStatus) -> &'static str {
    match status {
        ScanStatus::Pending => "pending",
        ScanStatus::Running => "running",
        ScanStatus::Completed => "completed",
        ScanStatus::Failed => "failed",
    }
}

fn technology_category_from_str(category: &str) -> TechnologyCategory {
    match category {
        "Language" => TechnologyCategory::Language,
        "Framework" => TechnologyCategory::Framework,
        "Database" => TechnologyCategory::Database,
        "ORM" => TechnologyCategory::Orm,
        "Package Manager" => TechnologyCategory::PackageManager,
        "Infrastructure" => TechnologyCategory::Infrastructure,
        _ => TechnologyCategory::Library,
    }
}

fn chat_message_role_from_str(role: &str) -> ChatMessageRole {
    match role {
        "User" => ChatMessageRole::User,
        "Assistant" => ChatMessageRole::Assistant,
        "System" => ChatMessageRole::System,
        _ => ChatMessageRole::User,
    }
}

fn sql_error(error: rusqlite::Error) -> DevAtlasError {
    DevAtlasError::new("storage.sqlite_error", error.to_string())
}

fn json_error(error: serde_json::Error) -> DevAtlasError {
    DevAtlasError::new("storage.json_error", error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{SqliteStorage, StorageService};
    use devatlas_common::{
        AiVectorEmbedding, AiVectorSnapshot, AiVectorSnapshotId, ChatMessage, ChatMessageId,
        ChatMessageRole, ChatSession, ChatSessionId, GraphEdge, GraphNode, GraphSnapshot,
        GraphSnapshotId, KnowledgeGraph, Repository, RepositoryId, RepositoryPath, ScanId,
        ScanResult, ScanStatus, Technology, TechnologyCategory,
    };

    #[test]
    fn mvp_schema_tracks_current_tables() {
        let plan = StorageService::mvp_plan();
        assert!(plan.schema.contains("repositories"));
        assert!(plan.schema.contains("graph_snapshots"));
        assert!(plan.schema.contains("ai_vector_snapshots"));
        assert!(plan.schema.contains("chat_sessions"));
        assert!(plan.schema.contains("chat_messages"));
        assert!(!plan.schema.contains("plugin_registry"));
    }

    #[test]
    fn persists_repository_scan_and_technologies() {
        let storage = SqliteStorage::open_in_memory().unwrap();
        let repository = Repository {
            id: RepositoryId("repo-1".to_string()),
            name: "sample".to_string(),
            path: RepositoryPath::new(std::env::temp_dir()).unwrap(),
            created_at_ms: 1,
        };
        storage.save_repository(&repository).unwrap();
        let scan = ScanResult {
            scan_id: ScanId("scan-1".to_string()),
            repository_id: repository.id.clone(),
            status: ScanStatus::Completed,
            files_count: 1,
            folders_count: 1,
            technologies: vec![Technology {
                category: TechnologyCategory::Language,
                name: "Rust".to_string(),
                version: None,
            }],
            files: Vec::new(),
            duration_ms: 5,
        };
        storage.save_scan(&scan).unwrap();

        assert_eq!(storage.list_repositories().unwrap().len(), 1);
        assert_eq!(storage.list_technologies("repo-1").unwrap()[0].name, "Rust");
    }

    #[test]
    fn persists_graph_snapshots() {
        let storage = SqliteStorage::open_in_memory().unwrap();
        let repository = Repository {
            id: RepositoryId("repo-1".to_string()),
            name: "sample".to_string(),
            path: RepositoryPath::new(std::env::temp_dir()).unwrap(),
            created_at_ms: 1,
        };
        storage.save_repository(&repository).unwrap();
        let graph = KnowledgeGraph {
            nodes: vec![GraphNode {
                id: "node-1".to_string(),
                node_type: "file".to_string(),
                name: "main.rs".to_string(),
            }],
            edges: vec![GraphEdge {
                id: "edge-1".to_string(),
                source: "node-1".to_string(),
                target: "node-1".to_string(),
                edge_type: "self".to_string(),
            }],
        };
        let snapshot = GraphSnapshot {
            id: GraphSnapshotId("snapshot-1".to_string()),
            repository_id: RepositoryId("repo-1".to_string()),
            scan_id: Some(ScanId("scan-1".to_string())),
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            graph,
            created_at_ms: 123,
        };

        storage.save_graph_snapshot(&snapshot).unwrap();
        let snapshots = storage.list_graph_snapshots("repo-1").unwrap();
        let loaded = storage.load_graph_snapshot("snapshot-1").unwrap();

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].node_count, 1);
        assert_eq!(loaded.graph.nodes[0].name, "main.rs");
        assert_eq!(loaded.scan_id, Some(ScanId("scan-1".to_string())));
    }

    #[test]
    fn persists_ai_vector_snapshots() {
        let storage = SqliteStorage::open_in_memory().unwrap();
        let repository = Repository {
            id: RepositoryId("repo-1".to_string()),
            name: "sample".to_string(),
            path: RepositoryPath::new(std::env::temp_dir()).unwrap(),
            created_at_ms: 1,
        };
        storage.save_repository(&repository).unwrap();
        let snapshot = AiVectorSnapshot {
            id: AiVectorSnapshotId("vector-snapshot-1".to_string()),
            repository_id: RepositoryId("repo-1".to_string()),
            scan_id: Some(ScanId("scan-1".to_string())),
            embeddings: vec![AiVectorEmbedding {
                id: "embedding-1".to_string(),
                chunk_id: "chunk-1".to_string(),
                path: "src/lib.rs".to_string(),
                dimensions: 4,
                model: "devatlas-local-hash-v1".to_string(),
                values: vec![0.5, 0.0, -0.5, 0.0],
            }],
            embedding_count: 1,
            dimensions: 4,
            model: "devatlas-local-hash-v1".to_string(),
            created_at_ms: 123,
        };

        storage.save_ai_vector_snapshot(&snapshot).unwrap();
        let snapshots = storage.list_ai_vector_snapshots("repo-1").unwrap();
        let loaded = storage
            .load_ai_vector_snapshot("vector-snapshot-1")
            .unwrap();

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].embedding_count, 1);
        assert_eq!(loaded.embeddings[0].chunk_id, "chunk-1");
        assert_eq!(loaded.scan_id, Some(ScanId("scan-1".to_string())));
    }

    #[test]
    fn persists_chat_sessions_and_messages() {
        let storage = SqliteStorage::open_in_memory().unwrap();
        let repository = Repository {
            id: RepositoryId("repo-1".to_string()),
            name: "sample".to_string(),
            path: RepositoryPath::new(std::env::temp_dir()).unwrap(),
            created_at_ms: 1,
        };
        storage.save_repository(&repository).unwrap();
        let session = ChatSession {
            id: ChatSessionId("chat-session-1".to_string()),
            repository_id: repository.id.clone(),
            title: "How does scanning work?".to_string(),
            created_at_ms: 100,
            updated_at_ms: 100,
        };
        storage.save_chat_session(&session).unwrap();

        let user_message = ChatMessage {
            id: ChatMessageId("chat-message-1".to_string()),
            repository_id: repository.id.clone(),
            session_id: session.id.clone(),
            role: ChatMessageRole::User,
            content: "How does scanning work?".to_string(),
            model: None,
            citation_count: 0,
            created_at_ms: 101,
        };
        let assistant_message = ChatMessage {
            id: ChatMessageId("chat-message-2".to_string()),
            repository_id: repository.id.clone(),
            session_id: session.id.clone(),
            role: ChatMessageRole::Assistant,
            content: "Scanning walks repository files.".to_string(),
            model: Some("devatlas-local-grounded-v1".to_string()),
            citation_count: 2,
            created_at_ms: 102,
        };
        storage.save_chat_message(&user_message).unwrap();
        storage.save_chat_message(&assistant_message).unwrap();

        let sessions = storage.list_chat_sessions("repo-1").unwrap();
        let loaded_session = storage.load_chat_session("chat-session-1").unwrap();
        let messages = storage.list_chat_messages("chat-session-1").unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].updated_at_ms, 102);
        assert_eq!(loaded_session.title, "How does scanning work?");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, ChatMessageRole::User);
        assert_eq!(messages[1].citation_count, 2);
    }
}
