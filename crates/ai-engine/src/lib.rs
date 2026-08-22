use devatlas_common::{stable_id, DevAtlasError, DevAtlasResult, RepositoryFile, RepositoryPath};
use std::collections::{HashMap, HashSet};
use std::fs;

const DEFAULT_MAX_CHARS: usize = 3_200;
const DEFAULT_OVERLAP_LINES: usize = 4;
const DEFAULT_MAX_FILE_BYTES: u64 = 512 * 1024;
const DEFAULT_EMBEDDING_DIMENSIONS: usize = 128;
const DEFAULT_RETRIEVAL_LIMIT: usize = 8;
const DEFAULT_CONTEXT_BUNDLE_MAX_TOKENS: usize = 4_000;
const LOCAL_CHAT_MODEL: &str = "devatlas-local-grounded-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextChunk {
    pub id: String,
    pub path: String,
    pub chunk_index: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub char_count: usize,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBuildOptions {
    pub max_chars: usize,
    pub overlap_lines: usize,
    pub max_file_bytes: u64,
}

impl Default for ContextBuildOptions {
    fn default() -> Self {
        Self {
            max_chars: DEFAULT_MAX_CHARS,
            overlap_lines: DEFAULT_OVERLAP_LINES,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBuildResult {
    pub chunks: Vec<ContextChunk>,
    pub skipped_files: Vec<SkippedContextFile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingVector {
    pub id: String,
    pub chunk_id: String,
    pub path: String,
    pub dimensions: usize,
    pub model: String,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingBuildOptions {
    pub dimensions: usize,
    pub model: String,
}

impl Default for EmbeddingBuildOptions {
    fn default() -> Self {
        Self {
            dimensions: DEFAULT_EMBEDDING_DIMENSIONS,
            model: "devatlas-local-hash-v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingBuildResult {
    pub embeddings: Vec<EmbeddingVector>,
    pub dimensions: usize,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorStoreBuildResult {
    pub embeddings: Vec<EmbeddingVector>,
    pub embedding_count: usize,
    pub dimensions: usize,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorStoreStats {
    pub embedding_count: usize,
    pub dimensions: usize,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalQuery {
    pub query: String,
    pub limit: usize,
}

impl RetrievalQuery {
    pub fn new(query: impl Into<String>, limit: Option<usize>) -> Self {
        Self {
            query: query.into(),
            limit: limit.unwrap_or(DEFAULT_RETRIEVAL_LIMIT),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalMatch {
    pub chunk_id: String,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub score: f32,
    pub vector_score: f32,
    pub lexical_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalResult {
    pub query: String,
    pub matches: Vec<RetrievalMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundleRequest {
    pub query: String,
    pub limit: usize,
    pub max_tokens: usize,
}

impl ContextBundleRequest {
    pub fn new(query: impl Into<String>, limit: Option<usize>, max_tokens: Option<usize>) -> Self {
        Self {
            query: query.into(),
            limit: limit.unwrap_or(DEFAULT_RETRIEVAL_LIMIT),
            max_tokens: max_tokens.unwrap_or(DEFAULT_CONTEXT_BUNDLE_MAX_TOKENS),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextBundleSource {
    pub source_id: String,
    pub chunk_id: String,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub score: f32,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextBundle {
    pub query: String,
    pub content: String,
    pub sources: Vec<ContextBundleSource>,
    pub token_estimate: usize,
    pub max_tokens: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRequest {
    pub question: String,
    pub limit: usize,
    pub max_context_tokens: usize,
}

impl ChatRequest {
    pub fn new(
        question: impl Into<String>,
        limit: Option<usize>,
        max_context_tokens: Option<usize>,
    ) -> Self {
        Self {
            question: question.into(),
            limit: limit.unwrap_or(DEFAULT_RETRIEVAL_LIMIT),
            max_context_tokens: max_context_tokens.unwrap_or(DEFAULT_CONTEXT_BUNDLE_MAX_TOKENS),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatCitation {
    pub source_id: String,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatResponse {
    pub question: String,
    pub answer: String,
    pub citations: Vec<ChatCitation>,
    pub context: ContextBundle,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedContextFile {
    pub path: String,
    pub reason: SkippedContextFileReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkippedContextFileReason {
    UnsupportedExtension,
    TooLarge,
    Unreadable,
    Empty,
}

impl SkippedContextFileReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedExtension => "Unsupported Extension",
            Self::TooLarge => "Too Large",
            Self::Unreadable => "Unreadable",
            Self::Empty => "Empty",
        }
    }
}

pub struct AiContextService;

pub struct AiEmbeddingService;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AiVectorStore {
    embeddings: Vec<EmbeddingVector>,
    dimensions: Option<usize>,
    model: Option<String>,
}

pub struct AiVectorStoreService;

pub struct AiRetrievalService;

pub struct AiContextBuilderService;

pub struct AiChatService;

impl AiContextService {
    pub fn build_context(
        repository_path: &RepositoryPath,
        files: &[RepositoryFile],
    ) -> DevAtlasResult<ContextBuildResult> {
        Self::build_context_with_options(repository_path, files, &ContextBuildOptions::default())
    }

    pub fn build_context_with_options(
        repository_path: &RepositoryPath,
        files: &[RepositoryFile],
        options: &ContextBuildOptions,
    ) -> DevAtlasResult<ContextBuildResult> {
        validate_options(options)?;

        let mut chunks = Vec::new();
        let mut skipped_files = Vec::new();

        for file in files.iter().filter(|file| is_context_source(file)) {
            if file.size_bytes > options.max_file_bytes {
                skipped_files.push(skipped_file(file, SkippedContextFileReason::TooLarge));
                continue;
            }

            let absolute_path = repository_path.as_path().join(&file.path);
            let Ok(content) = fs::read_to_string(absolute_path) else {
                skipped_files.push(skipped_file(file, SkippedContextFileReason::Unreadable));
                continue;
            };

            let file_chunks = chunk_file(&file.path, &content, options);
            if file_chunks.is_empty() {
                skipped_files.push(skipped_file(file, SkippedContextFileReason::Empty));
            } else {
                chunks.extend(file_chunks);
            }
        }

        for file in files.iter().filter(|file| !is_context_source(file)) {
            skipped_files.push(skipped_file(
                file,
                SkippedContextFileReason::UnsupportedExtension,
            ));
        }

        Ok(ContextBuildResult {
            chunks,
            skipped_files,
        })
    }
}

impl AiEmbeddingService {
    pub fn build_embeddings(context: &ContextBuildResult) -> DevAtlasResult<EmbeddingBuildResult> {
        Self::build_embeddings_with_options(context, &EmbeddingBuildOptions::default())
    }

    pub fn build_embeddings_with_options(
        context: &ContextBuildResult,
        options: &EmbeddingBuildOptions,
    ) -> DevAtlasResult<EmbeddingBuildResult> {
        validate_embedding_options(options)?;

        let embeddings = context
            .chunks
            .iter()
            .map(|chunk| build_embedding(chunk, options))
            .collect::<Vec<EmbeddingVector>>();

        Ok(EmbeddingBuildResult {
            embeddings,
            dimensions: options.dimensions,
            model: options.model.clone(),
        })
    }
}

impl AiVectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rebuild(&mut self, embeddings: &EmbeddingBuildResult) -> DevAtlasResult<()> {
        validate_embedding_result(embeddings)?;
        self.embeddings.clear();
        self.dimensions = Some(embeddings.dimensions);
        self.model = Some(embeddings.model.clone());
        for embedding in &embeddings.embeddings {
            self.upsert(embedding.clone())?;
        }
        Ok(())
    }

    pub fn upsert(&mut self, embedding: EmbeddingVector) -> DevAtlasResult<()> {
        self.validate_store_compatibility(&embedding)?;
        if let Some(position) = self
            .embeddings
            .iter()
            .position(|stored| stored.id == embedding.id)
        {
            self.embeddings[position] = embedding;
        } else {
            self.embeddings.push(embedding);
        }
        Ok(())
    }

    pub fn embeddings(&self) -> &[EmbeddingVector] {
        &self.embeddings
    }

    pub fn stats(&self) -> VectorStoreStats {
        VectorStoreStats {
            embedding_count: self.embeddings.len(),
            dimensions: self.dimensions.unwrap_or(0),
            model: self.model.clone().unwrap_or_default(),
        }
    }

    fn validate_store_compatibility(&mut self, embedding: &EmbeddingVector) -> DevAtlasResult<()> {
        if embedding.values.len() != embedding.dimensions {
            return Err(DevAtlasError::new(
                "ai_vector_store.embedding_dimension_mismatch",
                "Embedding values length must match its dimensions.",
            ));
        }

        match self.dimensions {
            Some(dimensions) if dimensions != embedding.dimensions => {
                return Err(DevAtlasError::new(
                    "ai_vector_store.dimensions_mismatch",
                    "Vector store cannot mix embeddings with different dimensions.",
                ));
            }
            None => self.dimensions = Some(embedding.dimensions),
            Some(_) => {}
        }

        match &self.model {
            Some(model) if model != &embedding.model => {
                return Err(DevAtlasError::new(
                    "ai_vector_store.model_mismatch",
                    "Vector store cannot mix embeddings from different models.",
                ));
            }
            None => self.model = Some(embedding.model.clone()),
            Some(_) => {}
        }

        Ok(())
    }
}

impl AiVectorStoreService {
    pub fn build_store(
        embeddings: &EmbeddingBuildResult,
    ) -> DevAtlasResult<VectorStoreBuildResult> {
        let mut store = AiVectorStore::new();
        store.rebuild(embeddings)?;
        Ok(VectorStoreBuildResult {
            embeddings: store.embeddings().to_vec(),
            embedding_count: store.stats().embedding_count,
            dimensions: embeddings.dimensions,
            model: embeddings.model.clone(),
        })
    }
}

impl AiRetrievalService {
    pub fn search(
        context: &ContextBuildResult,
        embeddings: &EmbeddingBuildResult,
        query: &RetrievalQuery,
    ) -> DevAtlasResult<RetrievalResult> {
        validate_retrieval_query(query)?;
        validate_embedding_result(embeddings)?;

        let embedding_options = EmbeddingBuildOptions {
            dimensions: embeddings.dimensions,
            model: embeddings.model.clone(),
        };
        let query_chunk = ContextChunk {
            id: stable_id("query", &query.query),
            path: "query".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
            content: query.query.clone(),
            char_count: query.query.chars().count(),
            token_estimate: estimate_tokens(&query.query),
        };
        let query_embedding = build_embedding(&query_chunk, &embedding_options);
        let chunks_by_id = context
            .chunks
            .iter()
            .map(|chunk| (chunk.id.as_str(), chunk))
            .collect::<HashMap<&str, &ContextChunk>>();

        let mut matches = embeddings
            .embeddings
            .iter()
            .filter_map(|embedding| {
                chunks_by_id
                    .get(embedding.chunk_id.as_str())
                    .map(|chunk| score_chunk(chunk, embedding, &query_embedding, &query.query))
            })
            .filter(|result| result.score > 0.0)
            .collect::<Vec<RetrievalMatch>>();

        matches.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.start_line.cmp(&right.start_line))
        });
        matches.truncate(query.limit);

        Ok(RetrievalResult {
            query: query.query.clone(),
            matches,
        })
    }
}

impl AiContextBuilderService {
    pub fn build_bundle(
        context: &ContextBuildResult,
        embeddings: &EmbeddingBuildResult,
        request: &ContextBundleRequest,
    ) -> DevAtlasResult<ContextBundle> {
        validate_context_bundle_request(request)?;
        let retrieval = AiRetrievalService::search(
            context,
            embeddings,
            &RetrievalQuery::new(&request.query, Some(request.limit)),
        )?;
        Ok(build_context_bundle_from_retrieval(
            &retrieval,
            request.max_tokens,
        ))
    }
}

impl AiChatService {
    pub fn answer(
        context: &ContextBuildResult,
        embeddings: &EmbeddingBuildResult,
        request: &ChatRequest,
    ) -> DevAtlasResult<ChatResponse> {
        validate_chat_request(request)?;
        let bundle = AiContextBuilderService::build_bundle(
            context,
            embeddings,
            &ContextBundleRequest::new(
                &request.question,
                Some(request.limit),
                Some(request.max_context_tokens),
            ),
        )?;
        let answer = build_grounded_answer(&request.question, &bundle);
        let citations = bundle
            .sources
            .iter()
            .map(|source| ChatCitation {
                source_id: source.source_id.clone(),
                path: source.path.clone(),
                start_line: source.start_line,
                end_line: source.end_line,
                score: source.score,
            })
            .collect();

        Ok(ChatResponse {
            question: request.question.clone(),
            answer,
            citations,
            context: bundle,
            model: LOCAL_CHAT_MODEL.to_string(),
        })
    }
}

fn validate_options(options: &ContextBuildOptions) -> DevAtlasResult<()> {
    if options.max_chars < 256 {
        return Err(DevAtlasError::new(
            "ai_context.max_chars_too_small",
            "AI context chunk size must be at least 256 characters.",
        ));
    }
    if options.overlap_lines > 32 {
        return Err(DevAtlasError::new(
            "ai_context.overlap_too_large",
            "AI context overlap cannot exceed 32 lines.",
        ));
    }
    if options.max_file_bytes == 0 {
        return Err(DevAtlasError::new(
            "ai_context.max_file_bytes_invalid",
            "AI context max file bytes must be greater than zero.",
        ));
    }
    Ok(())
}

fn validate_embedding_options(options: &EmbeddingBuildOptions) -> DevAtlasResult<()> {
    if options.dimensions < 16 {
        return Err(DevAtlasError::new(
            "ai_embedding.dimensions_too_small",
            "AI embedding dimensions must be at least 16.",
        ));
    }
    if options.dimensions > 4096 {
        return Err(DevAtlasError::new(
            "ai_embedding.dimensions_too_large",
            "AI embedding dimensions cannot exceed 4096.",
        ));
    }
    if options.model.trim().is_empty() {
        return Err(DevAtlasError::new(
            "ai_embedding.model_missing",
            "AI embedding model name is required.",
        ));
    }
    Ok(())
}

fn validate_embedding_result(embeddings: &EmbeddingBuildResult) -> DevAtlasResult<()> {
    for embedding in &embeddings.embeddings {
        if embedding.dimensions != embeddings.dimensions {
            return Err(DevAtlasError::new(
                "ai_vector_store.embedding_dimensions_invalid",
                "Embedding dimensions must match the embedding build result dimensions.",
            ));
        }
        if embedding.model != embeddings.model {
            return Err(DevAtlasError::new(
                "ai_vector_store.embedding_model_invalid",
                "Embedding model must match the embedding build result model.",
            ));
        }
        if embedding.values.len() != embeddings.dimensions {
            return Err(DevAtlasError::new(
                "ai_vector_store.embedding_values_invalid",
                "Embedding vector length must match the embedding dimensions.",
            ));
        }
    }
    Ok(())
}

fn validate_retrieval_query(query: &RetrievalQuery) -> DevAtlasResult<()> {
    if query.query.trim().is_empty() {
        return Err(DevAtlasError::new(
            "ai_retrieval.query_missing",
            "AI retrieval query is required.",
        ));
    }
    if query.limit == 0 {
        return Err(DevAtlasError::new(
            "ai_retrieval.limit_invalid",
            "AI retrieval limit must be greater than zero.",
        ));
    }
    if query.limit > 50 {
        return Err(DevAtlasError::new(
            "ai_retrieval.limit_too_large",
            "AI retrieval limit cannot exceed 50.",
        ));
    }
    Ok(())
}

fn validate_context_bundle_request(request: &ContextBundleRequest) -> DevAtlasResult<()> {
    validate_retrieval_query(&RetrievalQuery::new(&request.query, Some(request.limit)))?;
    if request.max_tokens < 128 {
        return Err(DevAtlasError::new(
            "ai_context_builder.max_tokens_too_small",
            "AI context bundle max tokens must be at least 128.",
        ));
    }
    if request.max_tokens > 64_000 {
        return Err(DevAtlasError::new(
            "ai_context_builder.max_tokens_too_large",
            "AI context bundle max tokens cannot exceed 64000.",
        ));
    }
    Ok(())
}

fn validate_chat_request(request: &ChatRequest) -> DevAtlasResult<()> {
    validate_context_bundle_request(&ContextBundleRequest::new(
        &request.question,
        Some(request.limit),
        Some(request.max_context_tokens),
    ))
}

fn is_context_source(file: &RepositoryFile) -> bool {
    matches!(
        file.extension.as_deref(),
        Some(
            "ts" | "tsx"
                | "js"
                | "jsx"
                | "rs"
                | "py"
                | "go"
                | "java"
                | "php"
                | "cs"
                | "md"
                | "json"
                | "toml"
                | "yaml"
                | "yml"
                | "sql"
                | "prisma"
        )
    )
}

fn chunk_file(path: &str, content: &str, options: &ContextBuildOptions) -> Vec<ContextChunk> {
    let lines = content.lines().collect::<Vec<&str>>();
    if lines.iter().all(|line| line.trim().is_empty()) {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start_index = 0;

    while start_index < lines.len() {
        let mut end_index = start_index;
        let mut char_count = 0;

        while end_index < lines.len() {
            let line_len = lines[end_index].chars().count() + 1;
            if end_index > start_index && char_count + line_len > options.max_chars {
                break;
            }
            char_count += line_len;
            end_index += 1;
        }

        if end_index == start_index {
            end_index += 1;
        }

        let content = lines[start_index..end_index].join("\n");
        let chunk_index = chunks.len();
        chunks.push(ContextChunk {
            id: stable_id(
                "chunk",
                &format!("{path}-{chunk_index}-{start_index}-{end_index}"),
            ),
            path: path.to_string(),
            chunk_index,
            start_line: start_index + 1,
            end_line: end_index,
            char_count: content.chars().count(),
            token_estimate: estimate_tokens(&content),
            content,
        });

        if end_index >= lines.len() {
            break;
        }
        start_index = end_index.saturating_sub(options.overlap_lines);
        if start_index == end_index {
            start_index += 1;
        }
    }

    chunks
}

fn estimate_tokens(content: &str) -> usize {
    (content.chars().count() / 4).max(1)
}

fn skipped_file(file: &RepositoryFile, reason: SkippedContextFileReason) -> SkippedContextFile {
    SkippedContextFile {
        path: file.path.clone(),
        reason,
    }
}

fn build_embedding(chunk: &ContextChunk, options: &EmbeddingBuildOptions) -> EmbeddingVector {
    let mut values = vec![0.0_f32; options.dimensions];

    for token in tokenize_for_embedding(&chunk.content) {
        let hash = hash_token(token);
        let index = (hash as usize) % options.dimensions;
        let sign = if hash & 1 == 0 { 1.0 } else { -1.0 };
        values[index] += sign;
    }

    normalize_vector(&mut values);

    EmbeddingVector {
        id: stable_id("embedding", &format!("{}-{}", options.model, chunk.id)),
        chunk_id: chunk.id.clone(),
        path: chunk.path.clone(),
        dimensions: options.dimensions,
        model: options.model.clone(),
        values,
    }
}

fn tokenize_for_embedding(content: &str) -> impl Iterator<Item = &str> {
    content
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
}

fn hash_token(token: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in token.to_ascii_lowercase().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

fn normalize_vector(values: &mut [f32]) {
    let magnitude = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude == 0.0 {
        return;
    }
    for value in values {
        *value /= magnitude;
    }
}

fn score_chunk(
    chunk: &ContextChunk,
    embedding: &EmbeddingVector,
    query_embedding: &EmbeddingVector,
    query: &str,
) -> RetrievalMatch {
    let vector_score = cosine_similarity(&embedding.values, &query_embedding.values).max(0.0);
    let lexical_score = lexical_overlap(query, &chunk.content);
    let score = (vector_score * 0.7) + (lexical_score * 0.3);

    RetrievalMatch {
        chunk_id: chunk.id.clone(),
        path: chunk.path.clone(),
        start_line: chunk.start_line,
        end_line: chunk.end_line,
        content: chunk.content.clone(),
        score,
        vector_score,
        lexical_score,
    }
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() {
        return 0.0;
    }
    left.iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| left_value * right_value)
        .sum::<f32>()
}

fn lexical_overlap(query: &str, content: &str) -> f32 {
    let query_tokens = normalized_token_set(query);
    let content_tokens = normalized_token_set(content);
    if query_tokens.is_empty() || content_tokens.is_empty() {
        return 0.0;
    }
    let overlap_count = query_tokens
        .iter()
        .filter(|token| content_tokens.contains(*token))
        .count();
    overlap_count as f32 / query_tokens.len() as f32
}

fn normalized_token_set(content: &str) -> HashSet<String> {
    tokenize_for_embedding(content)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn build_context_bundle_from_retrieval(
    retrieval: &RetrievalResult,
    max_tokens: usize,
) -> ContextBundle {
    let mut sections = Vec::new();
    let mut sources = Vec::new();
    let mut token_estimate = 0;
    let mut truncated = false;

    for (index, match_item) in retrieval.matches.iter().enumerate() {
        let source_id = format!("S{}", index + 1);
        let section = format!(
            "[{source_id}] {}:{}-{}\n{}",
            match_item.path, match_item.start_line, match_item.end_line, match_item.content
        );
        let section_tokens = estimate_tokens(&section);

        if token_estimate + section_tokens > max_tokens {
            truncated = true;
            break;
        }

        token_estimate += section_tokens;
        sections.push(section);
        sources.push(ContextBundleSource {
            source_id,
            chunk_id: match_item.chunk_id.clone(),
            path: match_item.path.clone(),
            start_line: match_item.start_line,
            end_line: match_item.end_line,
            score: match_item.score,
            token_estimate: section_tokens,
        });
    }

    ContextBundle {
        query: retrieval.query.clone(),
        content: sections.join("\n\n---\n\n"),
        sources,
        token_estimate,
        max_tokens,
        truncated,
    }
}

fn build_grounded_answer(question: &str, bundle: &ContextBundle) -> String {
    if bundle.sources.is_empty() {
        return format!(
            "I could not find repository context that answers this question: \"{}\".",
            question.trim()
        );
    }

    let source_list = bundle
        .sources
        .iter()
        .map(|source| {
            format!(
                "[{}] {}:{}-{}",
                source.source_id, source.path, source.start_line, source.end_line
            )
        })
        .collect::<Vec<String>>()
        .join(", ");
    let truncation_note = if bundle.truncated {
        " The context was truncated to fit the token budget."
    } else {
        ""
    };

    format!(
        "Based on the retrieved repository context, the most relevant sources are {source_list}. Review those snippets to answer: \"{}\".{truncation_note}",
        question.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AiChatService, AiContextBuilderService, AiContextService, AiEmbeddingService,
        AiRetrievalService, AiVectorStore, AiVectorStoreService, ChatRequest, ContextBuildOptions,
        ContextBundleRequest, EmbeddingBuildOptions, RetrievalQuery, SkippedContextFileReason,
    };
    use devatlas_common::{RepositoryFile, RepositoryPath};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn chunks_source_files_with_line_ranges() {
        let root = unique_temp_dir("devatlas-ai-context-chunks");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn one() {}\npub fn two() {}\npub fn three() {}\npub fn four() {}\n",
        )
        .unwrap();

        let repository_path = RepositoryPath::new(&root).unwrap();
        let result = AiContextService::build_context_with_options(
            &repository_path,
            &[file("src/lib.rs", 68)],
            &ContextBuildOptions {
                max_chars: 256,
                overlap_lines: 1,
                max_file_bytes: 1024,
            },
        )
        .unwrap();

        assert_eq!(result.chunks.len(), 1);
        assert_eq!(result.chunks[0].path, "src/lib.rs");
        assert_eq!(result.chunks[0].start_line, 1);
        assert_eq!(result.chunks[0].end_line, 4);
        assert!(result.chunks[0].token_estimate > 0);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn skips_unsupported_large_unreadable_and_empty_files() {
        let root = unique_temp_dir("devatlas-ai-context-skips");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/empty.ts"), "\n\n").unwrap();
        fs::write(root.join("src/invalid.ts"), [0xff, 0xfe]).unwrap();

        let repository_path = RepositoryPath::new(&root).unwrap();
        let result = AiContextService::build_context_with_options(
            &repository_path,
            &[
                file("src/empty.ts", 2),
                file("src/invalid.ts", 2),
                file("assets/logo.png", 12),
                file("src/large.ts", 2_048),
            ],
            &ContextBuildOptions {
                max_chars: 256,
                overlap_lines: 0,
                max_file_bytes: 1_024,
            },
        )
        .unwrap();

        assert!(result.chunks.is_empty());
        assert!(result.skipped_files.iter().any(|file| {
            file.path == "src/empty.ts" && file.reason == SkippedContextFileReason::Empty
        }));
        assert!(result.skipped_files.iter().any(|file| {
            file.path == "src/invalid.ts" && file.reason == SkippedContextFileReason::Unreadable
        }));
        assert!(result.skipped_files.iter().any(|file| {
            file.path == "assets/logo.png"
                && file.reason == SkippedContextFileReason::UnsupportedExtension
        }));
        assert!(result.skipped_files.iter().any(|file| {
            file.path == "src/large.ts" && file.reason == SkippedContextFileReason::TooLarge
        }));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn builds_deterministic_normalized_embeddings_for_chunks() {
        let root = unique_temp_dir("devatlas-ai-embeddings");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn scan_repository() {}\npub fn build_graph() {}\n",
        )
        .unwrap();

        let repository_path = RepositoryPath::new(&root).unwrap();
        let context = AiContextService::build_context_with_options(
            &repository_path,
            &[file("src/lib.rs", 55)],
            &ContextBuildOptions {
                max_chars: 256,
                overlap_lines: 0,
                max_file_bytes: 1024,
            },
        )
        .unwrap();
        let first = AiEmbeddingService::build_embeddings_with_options(
            &context,
            &EmbeddingBuildOptions {
                dimensions: 32,
                model: "devatlas-local-hash-v1".to_string(),
            },
        )
        .unwrap();
        let second = AiEmbeddingService::build_embeddings_with_options(
            &context,
            &EmbeddingBuildOptions {
                dimensions: 32,
                model: "devatlas-local-hash-v1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.embeddings.len(), 1);
        assert_eq!(first.embeddings[0].values.len(), 32);
        let magnitude = first.embeddings[0]
            .values
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        assert!((magnitude - 1.0).abs() < 0.0001);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn builds_vector_store_from_embeddings() {
        let embeddings = sample_embeddings();
        let store = AiVectorStoreService::build_store(&embeddings).unwrap();

        assert_eq!(store.embedding_count, 2);
        assert_eq!(store.dimensions, 16);
        assert_eq!(store.model, "devatlas-local-hash-v1");
        assert_eq!(store.embeddings.len(), 2);
    }

    #[test]
    fn vector_store_upserts_and_rejects_dimension_mismatch() {
        let mut store = AiVectorStore::new();
        let embeddings = sample_embeddings();
        store.upsert(embeddings.embeddings[0].clone()).unwrap();
        store.upsert(embeddings.embeddings[0].clone()).unwrap();

        assert_eq!(store.stats().embedding_count, 1);

        let mut invalid = embeddings.embeddings[1].clone();
        invalid.dimensions = 32;
        invalid.values = vec![0.0; 32];
        let error = store.upsert(invalid).unwrap_err();

        assert_eq!(error.code, "ai_vector_store.dimensions_mismatch");
    }

    #[test]
    fn hybrid_retrieval_ranks_matching_chunks() {
        let root = unique_temp_dir("devatlas-ai-retrieval");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/scanner.rs"),
            "pub fn scan_repository() {}\npub fn detect_technologies() {}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/export.rs"),
            "pub fn export_package() {}\npub fn write_zip() {}\n",
        )
        .unwrap();

        let repository_path = RepositoryPath::new(&root).unwrap();
        let context = AiContextService::build_context_with_options(
            &repository_path,
            &[file("src/scanner.rs", 62), file("src/export.rs", 53)],
            &ContextBuildOptions {
                max_chars: 256,
                overlap_lines: 0,
                max_file_bytes: 1024,
            },
        )
        .unwrap();
        let embeddings = AiEmbeddingService::build_embeddings_with_options(
            &context,
            &EmbeddingBuildOptions {
                dimensions: 32,
                model: "devatlas-local-hash-v1".to_string(),
            },
        )
        .unwrap();
        let result = AiRetrievalService::search(
            &context,
            &embeddings,
            &RetrievalQuery::new("scan technologies", Some(2)),
        )
        .unwrap();

        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].path, "src/scanner.rs");
        assert!(result.matches[0].lexical_score > 0.0);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retrieval_rejects_empty_query() {
        let error = AiRetrievalService::search(
            &super::ContextBuildResult {
                chunks: Vec::new(),
                skipped_files: Vec::new(),
            },
            &super::EmbeddingBuildResult {
                embeddings: Vec::new(),
                dimensions: 16,
                model: "devatlas-local-hash-v1".to_string(),
            },
            &RetrievalQuery::new(" ", Some(1)),
        )
        .unwrap_err();

        assert_eq!(error.code, "ai_retrieval.query_missing");
    }

    #[test]
    fn context_builder_creates_budgeted_source_bundle() {
        let root = unique_temp_dir("devatlas-ai-context-builder");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/scanner.rs"),
            "pub fn scan_repository() {}\npub fn detect_technologies() {}\n",
        )
        .unwrap();

        let repository_path = RepositoryPath::new(&root).unwrap();
        let context = AiContextService::build_context_with_options(
            &repository_path,
            &[file("src/scanner.rs", 62)],
            &ContextBuildOptions {
                max_chars: 256,
                overlap_lines: 0,
                max_file_bytes: 1024,
            },
        )
        .unwrap();
        let embeddings = AiEmbeddingService::build_embeddings_with_options(
            &context,
            &EmbeddingBuildOptions {
                dimensions: 32,
                model: "devatlas-local-hash-v1".to_string(),
            },
        )
        .unwrap();
        let bundle = AiContextBuilderService::build_bundle(
            &context,
            &embeddings,
            &ContextBundleRequest::new("scan technologies", Some(3), Some(512)),
        )
        .unwrap();

        assert_eq!(bundle.query, "scan technologies");
        assert_eq!(bundle.sources.len(), 1);
        assert!(bundle.content.contains("[S1] src/scanner.rs:1-2"));
        assert!(bundle.token_estimate <= 512);
        assert!(!bundle.truncated);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn context_builder_rejects_tiny_token_budget() {
        let error = AiContextBuilderService::build_bundle(
            &super::ContextBuildResult {
                chunks: Vec::new(),
                skipped_files: Vec::new(),
            },
            &super::EmbeddingBuildResult {
                embeddings: Vec::new(),
                dimensions: 16,
                model: "devatlas-local-hash-v1".to_string(),
            },
            &ContextBundleRequest::new("scan", Some(1), Some(64)),
        )
        .unwrap_err();

        assert_eq!(error.code, "ai_context_builder.max_tokens_too_small");
    }

    #[test]
    fn chat_backend_returns_grounded_answer_with_citations() {
        let root = unique_temp_dir("devatlas-ai-chat");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/scanner.rs"),
            "pub fn scan_repository() {}\npub fn detect_technologies() {}\n",
        )
        .unwrap();

        let repository_path = RepositoryPath::new(&root).unwrap();
        let context = AiContextService::build_context_with_options(
            &repository_path,
            &[file("src/scanner.rs", 62)],
            &ContextBuildOptions {
                max_chars: 256,
                overlap_lines: 0,
                max_file_bytes: 1024,
            },
        )
        .unwrap();
        let embeddings = AiEmbeddingService::build_embeddings_with_options(
            &context,
            &EmbeddingBuildOptions {
                dimensions: 32,
                model: "devatlas-local-hash-v1".to_string(),
            },
        )
        .unwrap();
        let response = AiChatService::answer(
            &context,
            &embeddings,
            &ChatRequest::new("How does scanning detect technologies?", Some(3), Some(512)),
        )
        .unwrap();

        assert_eq!(response.model, "devatlas-local-grounded-v1");
        assert_eq!(response.citations.len(), 1);
        assert!(response.answer.contains("[S1] src/scanner.rs:1-2"));
        assert!(response.context.content.contains("detect_technologies"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn chat_backend_rejects_empty_question() {
        let error = AiChatService::answer(
            &super::ContextBuildResult {
                chunks: Vec::new(),
                skipped_files: Vec::new(),
            },
            &super::EmbeddingBuildResult {
                embeddings: Vec::new(),
                dimensions: 16,
                model: "devatlas-local-hash-v1".to_string(),
            },
            &ChatRequest::new(" ", Some(1), Some(512)),
        )
        .unwrap_err();

        assert_eq!(error.code, "ai_retrieval.query_missing");
    }

    fn sample_embeddings() -> super::EmbeddingBuildResult {
        super::EmbeddingBuildResult {
            dimensions: 16,
            model: "devatlas-local-hash-v1".to_string(),
            embeddings: vec![
                super::EmbeddingVector {
                    id: "embedding-1".to_string(),
                    chunk_id: "chunk-1".to_string(),
                    path: "src/lib.rs".to_string(),
                    dimensions: 16,
                    model: "devatlas-local-hash-v1".to_string(),
                    values: vec![0.25; 16],
                },
                super::EmbeddingVector {
                    id: "embedding-2".to_string(),
                    chunk_id: "chunk-2".to_string(),
                    path: "src/main.rs".to_string(),
                    dimensions: 16,
                    model: "devatlas-local-hash-v1".to_string(),
                    values: vec![0.0; 16],
                },
            ],
        }
    }

    fn file(path: &str, size_bytes: u64) -> RepositoryFile {
        RepositoryFile {
            path: path.to_string(),
            extension: path.rsplit('.').next().map(ToString::to_string),
            size_bytes,
        }
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("{prefix}-{timestamp}"))
    }
}
