use devatlas_common::{
    stable_id, DocumentId, DocumentType, DocumentationPlan, DocumentationQuality,
    DocumentationSectionPlan, GeneratedDocument, KnowledgeGraph, ScanResult,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct DocumentationService;

impl DocumentationService {
    pub fn generate_documents(scan: &ScanResult, graph: &KnowledgeGraph) -> Vec<GeneratedDocument> {
        let context = build_semantic_context(scan, graph);
        vec![
            GeneratedDocument {
                id: DocumentId(stable_id("doc", "readme")),
                path: "docs/README.md".to_string(),
                document_type: DocumentType::Readme,
                content: generate_readme(scan, &context),
                semantic_plan: readme_plan(),
                quality: context.quality_for("README"),
            },
            GeneratedDocument {
                id: DocumentId(stable_id("doc", "architecture")),
                path: "docs/architecture.md".to_string(),
                document_type: DocumentType::Architecture,
                content: generate_architecture(scan, graph, &context),
                semantic_plan: architecture_plan(),
                quality: context.quality_for("Architecture"),
            },
            GeneratedDocument {
                id: DocumentId(stable_id("doc", "developer-guide")),
                path: "docs/developer-guide.md".to_string(),
                document_type: DocumentType::Modules,
                content: generate_developer_guide(scan, graph, &context),
                semantic_plan: developer_guide_plan(),
                quality: context.quality_for("Developer Guide"),
            },
            GeneratedDocument {
                id: DocumentId(stable_id("doc", "api-summary")),
                path: "docs/api-summary.md".to_string(),
                document_type: DocumentType::ApiSummary,
                content: generate_api_summary(graph, &context),
                semantic_plan: api_plan(),
                quality: context.quality_for("API Summary"),
            },
            GeneratedDocument {
                id: DocumentId(stable_id("doc", "database-summary")),
                path: "docs/database-summary.md".to_string(),
                document_type: DocumentType::DatabaseSummary,
                content: generate_database_summary(scan, &context),
                semantic_plan: database_plan(),
                quality: context.quality_for("Database Summary"),
            },
            GeneratedDocument {
                id: DocumentId(stable_id("doc", "onboarding")),
                path: "docs/onboarding.md".to_string(),
                document_type: DocumentType::Onboarding,
                content: generate_onboarding(scan, graph, &context),
                semantic_plan: onboarding_plan(),
                quality: context.quality_for("Onboarding"),
            },
            GeneratedDocument {
                id: DocumentId(stable_id("doc", "ai-context")),
                path: "docs/ai-context.md".to_string(),
                document_type: DocumentType::AiContext,
                content: generate_ai_context(scan, graph, &context),
                semantic_plan: ai_context_plan(),
                quality: context.quality_for("AI Context"),
            },
        ]
    }
}

#[derive(Debug, Clone)]
struct SemanticContext {
    clusters: Vec<SemanticCluster>,
    entrypoints: Vec<String>,
    api_files: Vec<String>,
    database_files: Vec<String>,
    important_files: Vec<String>,
    symbols: Vec<String>,
    dependencies: Vec<String>,
    modules: Vec<String>,
    evidence_sources: Vec<String>,
    warnings: Vec<String>,
    source_count: usize,
    symbol_count: usize,
    graph_edge_count: usize,
}

#[derive(Debug, Clone)]
struct SemanticCluster {
    name: &'static str,
    count: usize,
    examples: Vec<String>,
}

impl SemanticContext {
    fn quality_for(&self, document_name: &str) -> DocumentationQuality {
        let file_score = percentile_score(self.source_count, 10, 150, 35);
        let technology_score = percentile_score(self.evidence_sources.len(), 1, 8, 15);
        let edge_score = percentile_score(self.graph_edge_count, 5, 250, 25);
        let symbol_score = percentile_score(self.symbol_count, 3, 100, 25);
        let cluster_score = percentile_score(self.clusters.len(), 1, 6, 20);
        let coverage_score = clamp_u8(15 + file_score + technology_score + edge_score, 0, 100);
        let semantic_score = clamp_u8(10 + symbol_score + cluster_score + edge_score, 0, 100);
        let mut warnings = self.warnings.clone();
        if document_name == "API Summary" && self.api_files.is_empty() {
            warnings.push("No API entrypoint files were detected by path heuristics.".to_string());
        }
        if document_name == "Database Summary" && self.database_files.is_empty() {
            warnings.push("No database or schema files were detected by path heuristics.".to_string());
        }
        DocumentationQuality {
            coverage_score,
            semantic_score,
            source_count: self.source_count,
            symbol_count: self.symbol_count,
            graph_edge_count: self.graph_edge_count,
            warnings,
        }
    }
}

fn build_semantic_context(scan: &ScanResult, graph: &KnowledgeGraph) -> SemanticContext {
    let mut clusters: BTreeMap<&'static str, (usize, BTreeSet<String>)> = BTreeMap::new();
    let mut entrypoints = BTreeSet::new();
    let mut api_files = BTreeSet::new();
    let mut database_files = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    let mut dependencies = BTreeSet::new();
    let mut modules = BTreeSet::new();
    let mut warnings = Vec::new();

    for file in &scan.files {
        if let Some(cluster) = classify_cluster(&file.path, "File") {
            let entry = clusters.entry(cluster).or_default();
            entry.0 += 1;
            entry.1.insert(file.path.clone());
        }
        if is_api_path(&file.path) {
            api_files.insert(file.path.clone());
            entrypoints.insert(file.path.clone());
        }
        if is_database_path(&file.path) {
            database_files.insert(file.path.clone());
        }
        if is_entrypoint_path(&file.path) {
            entrypoints.insert(file.path.clone());
        }
    }

    for node in &graph.nodes {
        if is_symbol_type(&node.node_type) {
            symbols.insert(format!("{} in `{}`", node.node_type, node.name));
        }
        if node.node_type == "Module" {
            modules.insert(node.name.clone());
            if let Some(cluster) = classify_cluster(&node.name, "Module") {
                let entry = clusters.entry(cluster).or_default();
                entry.0 += 1;
                entry.1.insert(node.name.clone());
            }
        }
        if node.node_type == "Dependency" {
            dependencies.insert(node.name.clone());
        }
        if let Some(cluster) = classify_cluster(&node.name, &node.node_type) {
            let entry = clusters.entry(cluster).or_default();
            entry.0 += 1;
            entry.1.insert(node.name.clone());
        }
    }

    let important_files = scan
        .files
        .iter()
        .map(|file| (file.size_bytes, file.path.clone()))
        .collect::<Vec<(u64, String)>>()
        .into_iter()
        .rev()
        .take(30)
        .map(|(_, path)| path)
        .collect::<Vec<String>>();

    if scan.files.is_empty() {
        warnings.push("No source files were available for documentation evidence.".to_string());
    }
    if scan.technologies.is_empty() {
        warnings.push("No technologies were detected.".to_string());
    }
    if graph.edges.is_empty() {
        warnings.push("No graph edges were available for dependency interpretation.".to_string());
    }
    if symbols.is_empty() {
        warnings.push("No parsed symbols were available for API or module explanation.".to_string());
    }

    let clusters = clusters
        .into_iter()
        .map(|(name, (count, examples))| SemanticCluster {
            name,
            count,
            examples: examples.into_iter().take(6).collect(),
        })
        .collect::<Vec<SemanticCluster>>();

    let evidence_sources = vec![
        format!("{} scanned files", scan.files_count),
        format!("{} scanned folders", scan.folders_count),
        format!("{} detected technologies", scan.technologies.len()),
        format!("{} graph nodes", graph.nodes.len()),
        format!("{} graph edges", graph.edges.len()),
        format!("{} semantic clusters", clusters.len()),
        format!("{} parsed symbols", symbols.len()),
    ];

    let symbol_count = symbols.len();
    let graph_edge_count = graph.edges.len();

    SemanticContext {
        clusters,
        entrypoints: entrypoints.into_iter().take(30).collect(),
        api_files: api_files.into_iter().take(80).collect(),
        database_files: database_files.into_iter().take(80).collect(),
        important_files,
        symbols: symbols.into_iter().take(120).collect(),
        dependencies: dependencies.into_iter().take(100).collect(),
        modules: modules.into_iter().take(80).collect(),
        evidence_sources,
        warnings,
        source_count: scan.files.len(),
        symbol_count,
        graph_edge_count,
    }
}

fn generate_readme(scan: &ScanResult, context: &SemanticContext) -> String {
    format!(
        "# Repository Knowledge Package\n\n## Evidence-Based Summary\n\nThis document is generated without AI assistance from the local scan, technology detection, and knowledge graph. It summarizes what DevAtlas can prove from the repository structure and graph signals.\n\n## Repository Scale\n\n- Files: {}\n- Folders: {}\n- Scan Duration: {} ms\n- Graph Nodes: built from scan and parser evidence\n\n## Technology Stack\n\n{}\n\n## Semantic Clusters\n\n{}\n\n## Likely Entrypoints\n\n{}\n\n## Documentation Guardrails\n\n- Claims are based on scan files, detected technologies, graph nodes, graph edges, and parsed symbols.\n- Deployment, secrets, runtime behavior, and undocumented endpoints are not inferred unless explicit evidence exists.\n- Security and performance sections remain out of scope until dedicated engines provide evidence.\n",
        scan.files_count,
        scan.folders_count,
        scan.duration_ms,
        list_or_empty(format_technologies(scan), "No technologies were detected."),
        list_or_empty(format_clusters(&context.clusters), "No semantic clusters were discovered."),
        list_or_empty(format_files(&context.entrypoints), "No entrypoint files were detected by path heuristics.")
    )
}

fn generate_architecture(scan: &ScanResult, graph: &KnowledgeGraph, context: &SemanticContext) -> String {
    format!(
        "# Architecture Overview\n\n## Evidence Basis\n\n- Files: {}\n- Folders: {}\n- Graph Nodes: {}\n- Graph Edges: {}\n- Semantic Clusters: {}\n\n## Semantic Cluster Map\n\n{}\n\n## Dependency Flow\n\nThe graph is interpreted as repository-to-module, module-to-file, file-to-symbol, and file-to-dependency relationships. Direct dependency nodes are listed below when the parser detected imports or relationships.\n\n{}\n\n## Module and Symbol Signals\n\n### Modules\n\n{}\n\n### Symbols\n\n{}\n\n## Architecture Notes\n\n- Cluster counts explain which parts of the repository are most represented in the scan.\n- Entrypoint files identify likely places where runtime or request handling starts.\n- Dependency nodes are treated as external or unresolved references, not confirmed runtime wiring.\n- Missing symbols or edges are reported as documentation warnings rather than invented details.\n",
        scan.files_count,
        scan.folders_count,
        graph.nodes.len(),
        graph.edges.len(),
        context.clusters.len(),
        list_or_empty(format_clusters(&context.clusters), "No semantic clusters were discovered."),
        list_or_empty(format_dependencies(&context.dependencies), "No external import dependencies were discovered."),
        list_or_empty(format_files(&context.modules), "No module nodes were discovered."),
        list_or_empty(format_symbol_lines(&context.symbols), "No parsed symbols were discovered.")
    )
}

fn generate_developer_guide(scan: &ScanResult, graph: &KnowledgeGraph, context: &SemanticContext) -> String {
    format!(
        "# Developer Guide\n\n## Where to Start\n\nThis guide is deterministic and evidence-backed. Use it to understand where to inspect or change code first.\n\n## Repository Scale\n\n- Files: {}\n- Folders: {}\n- Graph Nodes: {}\n- Graph Edges: {}\n\n## Semantic Clusters\n\n{}\n\n## Candidate Entrypoints\n\n{}\n\n## API and Database Files\n\n### API Candidates\n\n{}\n\n### Database Candidates\n\n{}\n\n## Parsed Symbols\n\n{}\n\n## Suggested Workflow\n\n1. Start with the semantic cluster that matches the feature area.\n2. Open the candidate entrypoint files before tracing lower-level modules.\n3. Use parsed symbols to find concrete classes, functions, methods, structs, interfaces, or traits.\n4. Treat dependency nodes as unresolved import signals unless the parser provides stronger evidence.\n5. Use warnings as documentation debt, not as confirmed repository defects.\n",
        scan.files_count,
        scan.folders_count,
        graph.nodes.len(),
        graph.edges.len(),
        list_or_empty(format_clusters(&context.clusters), "No semantic clusters were discovered."),
        list_or_empty(format_files(&context.entrypoints), "No entrypoint files were detected by path heuristics."),
        list_or_empty(format_files(&context.api_files), "No API-related files were detected by path heuristics."),
        list_or_empty(format_files(&context.database_files), "No database or schema files were detected by path heuristics."),
        list_or_empty(format_symbol_lines(&context.symbols), "No parsed symbols were discovered.")
    )
}

fn generate_api_summary(graph: &KnowledgeGraph, context: &SemanticContext) -> String {
    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<&str, &devatlas_common::GraphNode>>();
    let symbols = graph
        .edges
        .iter()
        .filter(|edge| edge.edge_type == "Contains")
        .filter_map(|edge| {
            let source = nodes_by_id.get(edge.source.as_str())?;
            let target = nodes_by_id.get(edge.target.as_str())?;
            if source.node_type == "File"
                && context.api_files.iter().any(|api_file| api_file == &source.name)
                && is_symbol_type(&target.node_type)
            {
                Some(format!("{} `{}` in `{}`", target.node_type, target.name, source.name))
            } else {
                None
            }
        })
        .take(120)
        .collect::<Vec<String>>();

    format!(
        "# API Summary\n\n## Evidence Basis\n\nAPI candidates are selected from paths containing `api`, `route`, `controller`, `handler`, or common backend entrypoint names. Symbols are included only when a graph `Contains` edge connects an API candidate file to a parsed symbol.\n\n## Candidate API Files\n\n{}\n\n## Entrypoints\n\n{}\n\n## API Symbols\n\n{}\n\n## Interpretation Guardrails\n\n- This document does not invent HTTP methods, routes, request bodies, or response schemas.\n- Controller, route, or handler files are candidates, not confirmed public endpoints.\n- If no symbols are listed, the parser did not expose enough symbol evidence for API explanation.\n",
        list_or_empty(format_files(&context.api_files), "No API-related files were detected by path heuristics."),
        list_or_empty(format_files(&context.entrypoints), "No entrypoint files were detected by path heuristics."),
        list_or_empty(symbols, "No API-related symbols were discovered in candidate files.")
    )
}

fn generate_database_summary(scan: &ScanResult, context: &SemanticContext) -> String {
    let database_technologies = scan
        .technologies
        .iter()
        .filter(|technology| {
            let category = technology.category.as_str();
            category == "Database" || category == "ORM"
        })
        .map(|technology| format!("{}: {}", technology.category.as_str(), technology.name))
        .collect::<Vec<String>>();

    format!(
        "# Database Summary\n\n## Evidence Basis\n\nDatabase evidence is based on detected database/ORM technologies and paths containing schema, migration, `.prisma`, `.sql`, `repository`, `persistence`, or `db` signals.\n\n## Database and ORM Technologies\n\n{}\n\n## Candidate Schema or Persistence Files\n\n{}\n\n## Important Large Files That May Contain Data Logic\n\n{}\n\n## Interpretation Guardrails\n\n- This document does not infer table names, relations, migrations, or SQL semantics unless explicit files are detected.\n- Repository or persistence paths are candidates for data access code, not confirmed ORM mappings.\n- Runtime connection strings, credentials, and deployment configuration are intentionally excluded.\n",
        list_or_empty(database_technologies, "No database or ORM technologies were detected."),
        list_or_empty(format_files(&context.database_files), "No schema, migration, or database-related files were detected."),
        list_or_empty(format_files(&context.important_files), "No large source files were available for data logic review.")
    )
}

fn generate_onboarding(scan: &ScanResult, graph: &KnowledgeGraph, context: &SemanticContext) -> String {
    format!(
        "# Developer Onboarding\n\n## Repository Scale\n\n- Files: {}\n- Folders: {}\n- Graph Nodes: {}\n- Graph Edges: {}\n\n## Technology Stack\n\n{}\n\n## Semantic Clusters\n\n{}\n\n## First Files to Inspect\n\n{}\n\n## API and Database Entry Points\n\n{}\n\n## Parsed Symbols\n\n{}\n\n## Suggested Workflow\n\n1. Read the README summary to understand the evidence basis.\n2. Open the top entrypoint files that match the area you need to change.\n3. Review semantic clusters to understand the repository shape.\n4. Use API/database candidate files for feature work involving endpoints or persistence.\n5. Treat warnings as documentation gaps to verify manually.\n\n## Generation Guardrails\n\nThis onboarding guide is generated from local scan results and the internal knowledge graph only. Git, security, performance, cloud, and runtime deployment claims remain out of scope until dedicated engines provide proven evidence.\n",
        scan.files_count,
        scan.folders_count,
        graph.nodes.len(),
        graph.edges.len(),
        list_or_empty(format_technologies(scan), "No technologies were detected."),
        list_or_empty(format_clusters(&context.clusters), "No semantic clusters were discovered."),
        list_or_empty(format_files(&context.entrypoints), "No entrypoint files were detected by path heuristics."),
        list_or_empty(
            [context.api_files.clone(), context.database_files.clone()].concat(),
            "No API or database entry point files were detected."
        ),
        list_or_empty(format_symbol_lines(&context.symbols), "No parsed symbols were discovered.")
    )
}

fn generate_ai_context(scan: &ScanResult, graph: &KnowledgeGraph, context: &SemanticContext) -> String {
    format!(
        "# AI Context\n\n## Repository Summary\n\n- Repository ID: `{}`\n- Scan ID: `{}`\n- Files: {}\n- Folders: {}\n- Scan Duration: {} ms\n\n## Architecture Signals\n\n- Graph Nodes: {}\n- Graph Edges: {}\n- Semantic Clusters: {}\n- Parsed Symbols: {}\n\n## Evidence Sources\n\n{}\n\n## Technologies\n\n{}\n\n## Important Files\n\n{}\n\n## Semantic Clusters\n\n{}\n\n## API Candidates\n\n{}\n\n## Database Candidates\n\n{}\n\n## Module Signals\n\n{}\n\n## Dependency Signals\n\n{}\n\n## Context Rules\n\nUse this context as a deterministic repository summary. Do not assume undocumented endpoints, database relations, secrets, runtime behavior, deployment state, security posture, or performance characteristics unless separate DevAtlas engines provide those facts. Treat warnings as missing evidence.\n",
        scan.repository_id.0.as_str(),
        scan.scan_id.0.as_str(),
        scan.files_count,
        scan.folders_count,
        scan.duration_ms,
        graph.nodes.len(),
        graph.edges.len(),
        context.clusters.len(),
        context.symbol_count,
        list_or_empty(context.evidence_sources.clone(), "No evidence sources were available."),
        list_or_empty(format_technologies(scan), "No technologies were detected."),
        list_or_empty(format_files(&context.important_files), "No source files were detected."),
        list_or_empty(format_clusters(&context.clusters), "No semantic clusters were discovered."),
        list_or_empty(format_files(&context.api_files), "No API-related files were detected by path heuristics."),
        list_or_empty(format_files(&context.database_files), "No database or schema files were detected by path heuristics."),
        list_or_empty(format_files(&context.modules), "No module nodes were discovered."),
        list_or_empty(format_dependencies(&context.dependencies), "No dependency nodes were discovered.")
    )
}

fn readme_plan() -> DocumentationPlan {
    DocumentationPlan {
        audience: "Repository readers and reviewers".to_string(),
        intent: "Summarize proven repository facts before deeper documentation".to_string(),
        sections: vec![
            section("Evidence-Based Summary", "Explain what the document is based on", "Scan and graph metadata", vec!["files", "folders", "graph nodes"]),
            section("Technology Stack", "List detected technologies", "Technology detection", vec!["technology categories", "technology names"]),
            section("Semantic Clusters", "Group repository areas by path and graph signals", "Graph and file path classification", vec!["clusters", "module paths", "file paths"]),
            section("Likely Entrypoints", "Identify candidate start files", "Path heuristics", vec!["api", "route", "controller", "handler", "main"]),
        ],
        evidence_sources: vec![
            "scan.files".to_string(),
            "scan.technologies".to_string(),
            "knowledge graph nodes".to_string(),
            "knowledge graph edges".to_string(),
        ],
    }
}

fn architecture_plan() -> DocumentationPlan {
    DocumentationPlan {
        audience: "Architects and senior developers".to_string(),
        intent: "Explain repository structure and dependency flow from graph evidence".to_string(),
        sections: vec![
            section("Evidence Basis", "State measurable graph and scan coverage", "Scan and graph metadata", vec!["files", "folders", "nodes", "edges"]),
            section("Semantic Cluster Map", "Describe dominant repository areas", "Cluster classification", vec!["cluster count", "cluster examples"]),
            section("Dependency Flow", "List external or unresolved dependency signals", "Graph relationship edges", vec!["Dependency nodes", "relationship edges"]),
            section("Module and Symbol Signals", "Expose structural and symbol evidence", "Graph nodes", vec!["Module nodes", "symbol nodes"]),
        ],
        evidence_sources: vec![
            "knowledge graph nodes".to_string(),
            "knowledge graph edges".to_string(),
            "scan files".to_string(),
            "parsed symbols".to_string(),
        ],
    }
}

fn developer_guide_plan() -> DocumentationPlan {
    DocumentationPlan {
        audience: "Developers changing the repository".to_string(),
        intent: "Tell developers where to start and which files matter".to_string(),
        sections: vec![
            section("Where to Start", "Provide deterministic onboarding guidance", "Scan and graph metadata", vec!["files", "folders", "graph signals"]),
            section("Candidate Entrypoints", "List files likely to start runtime or request flow", "Path heuristics", vec!["api", "route", "controller", "handler", "main"]),
            section("API and Database Files", "Separate endpoint and persistence candidates", "Path heuristics", vec!["api paths", "database paths"]),
            section("Parsed Symbols", "List concrete symbols for navigation", "Parser graph edges", vec!["Function", "Class", "Struct", "Interface", "Trait"]),
        ],
        evidence_sources: vec![
            "scan files".to_string(),
            "graph nodes".to_string(),
            "graph edges".to_string(),
            "parser symbols".to_string(),
        ],
    }
}

fn api_plan() -> DocumentationPlan {
    DocumentationPlan {
        audience: "Backend and integration developers".to_string(),
        intent: "Describe API candidates without inventing endpoints".to_string(),
        sections: vec![
            section("Evidence Basis", "Explain API candidate selection rules", "Path and graph evidence", vec!["api paths", "Contains edges"]),
            section("Candidate API Files", "List files that look like API entrypoints", "Path heuristics", vec!["api", "route", "controller", "handler"]),
            section("API Symbols", "List symbols contained by API files", "Graph Contains edges", vec!["File", "Function", "Class", "Method"]),
            section("Interpretation Guardrails", "State what is not inferred", "Documentation policy", vec!["HTTP method", "request body", "response schema"]),
        ],
        evidence_sources: vec![
            "api candidate paths".to_string(),
            "graph Contains edges".to_string(),
            "parsed symbols".to_string(),
        ],
    }
}

fn database_plan() -> DocumentationPlan {
    DocumentationPlan {
        audience: "Backend developers and data maintainers".to_string(),
        intent: "Summarize database and persistence evidence without inventing schemas".to_string(),
        sections: vec![
            section("Evidence Basis", "Explain database path and technology rules", "Scan and graph evidence", vec!["database technologies", "schema paths"]),
            section("Database and ORM Technologies", "List detected data technologies", "Technology detection", vec!["Database", "ORM"]),
            section("Candidate Schema or Persistence Files", "List schema, migration, repository, and db files", "Path heuristics", vec!["schema", "migration", ".prisma", ".sql", "db"]),
            section("Interpretation Guardrails", "State what is not inferred", "Documentation policy", vec!["table names", "relations", "credentials"]),
        ],
        evidence_sources: vec![
            "scan technologies".to_string(),
            "scan files".to_string(),
            "graph nodes".to_string(),
        ],
    }
}

fn onboarding_plan() -> DocumentationPlan {
    DocumentationPlan {
        audience: "New developers".to_string(),
        intent: "Provide a safe first-pass workflow for understanding the repository".to_string(),
        sections: vec![
            section("Repository Scale", "Give measurable size context", "Scan metadata", vec!["files", "folders", "nodes", "edges"]),
            section("Technology Stack", "List detected stack signals", "Technology detection", vec!["language", "framework", "database", "package manager"]),
            section("Semantic Clusters", "Explain repository areas", "Cluster classification", vec!["ui", "api", "domain", "infrastructure", "tests"]),
            section("First Files to Inspect", "Suggest concrete starting points", "Path heuristics", vec!["entrypoints", "api files", "database files"]),
        ],
        evidence_sources: vec![
            "scan files".to_string(),
            "scan technologies".to_string(),
            "graph nodes".to_string(),
            "graph edges".to_string(),
        ],
    }
}

fn ai_context_plan() -> DocumentationPlan {
    DocumentationPlan {
        audience: "AI-assisted tools and future context consumers".to_string(),
        intent: "Provide deterministic evidence for downstream reasoning".to_string(),
        sections: vec![
            section("Repository Summary", "Provide identifiers and scan metrics", "Scan metadata", vec!["repository id", "scan id", "files", "folders"]),
            section("Architecture Signals", "Expose graph coverage", "Knowledge graph", vec!["nodes", "edges", "clusters", "symbols"]),
            section("Important Files", "List high-value files for context", "File size and scan order", vec!["source files", "large files"]),
            section("Context Rules", "Prevent unsupported inference", "Documentation policy", vec!["no secrets", "no runtime assumptions", "no deployment claims"]),
        ],
        evidence_sources: vec![
            "scan metadata".to_string(),
            "knowledge graph".to_string(),
            "semantic clusters".to_string(),
            "parsed symbols".to_string(),
            "dependency nodes".to_string(),
        ],
    }
}

fn section(title: &str, purpose: &str, evidence_type: &str, required_signals: Vec<&str>) -> DocumentationSectionPlan {
    DocumentationSectionPlan {
        title: title.to_string(),
        purpose: purpose.to_string(),
        evidence_type: evidence_type.to_string(),
        required_signals: required_signals.into_iter().map(|signal| signal.to_string()).collect(),
    }
}

fn format_technologies(scan: &ScanResult) -> Vec<String> {
    scan.technologies
        .iter()
        .map(|technology| {
            let version = technology
                .version
                .as_ref()
                .map(|version| format!(" ({version})"))
                .unwrap_or_default();
            format!("{}: {}{}", technology.category.as_str(), technology.name, version)
        })
        .collect()
}

fn format_clusters(clusters: &[SemanticCluster]) -> Vec<String> {
    clusters
        .iter()
        .map(|cluster| {
            let examples = cluster
                .examples
                .iter()
                .map(|example| format!("`{example}`"))
                .collect::<Vec<String>>()
                .join(", ");
            format!("{} ({}): {}", cluster.name, cluster.count, examples)
        })
        .collect()
}

fn format_files(files: &[String]) -> Vec<String> {
    files.iter().map(|file| format!("`{file}`")).collect()
}

fn format_dependencies(dependencies: &[String]) -> Vec<String> {
    dependencies.iter().map(|dependency| format!("`{dependency}`")).collect()
}

fn format_symbol_lines(symbols: &[String]) -> Vec<String> {
    symbols.iter().map(|symbol| format!("- {symbol}")).collect()
}

fn list_or_empty(items: Vec<String>, empty_message: &str) -> String {
    if items.is_empty() {
        empty_message.to_string()
    } else {
        items.join("\n")
    }
}

fn is_database_path(path: &str) -> bool {
    let lower_path = path.to_lowercase();
    lower_path.contains("schema")
        || lower_path.contains("migration")
        || lower_path.ends_with(".prisma")
        || lower_path.ends_with(".sql")
        || lower_path.contains("repository")
        || lower_path.contains("persistence")
        || lower_path.contains("/db/")
        || lower_path.contains("\\db\\")
}

fn is_api_path(path: &str) -> bool {
    let lower_path = path.to_lowercase();
    lower_path.contains("api")
        || lower_path.contains("route")
        || lower_path.contains("controller")
        || lower_path.contains("handler")
}

fn is_entrypoint_path(path: &str) -> bool {
    let lower_path = path.to_lowercase();
    lower_path == "src/main.rs"
        || lower_path == "src/main.ts"
        || lower_path == "src/main.tsx"
        || lower_path == "src/index.ts"
        || lower_path == "src/index.tsx"
        || lower_path == "src/app/page.tsx"
        || lower_path.starts_with("src/app/")
        || lower_path.starts_with("src/pages/")
        || lower_path.starts_with("app/")
        || lower_path.starts_with("bin/")
        || is_api_path(&lower_path)
}

fn classify_cluster(path_or_name: &str, node_type: &str) -> Option<&'static str> {
    let normalized = path_or_name.to_lowercase().replace('\\', "/");
    if node_type == "Dependency" {
        return Some("External");
    }
    if normalized.contains("test") || normalized.contains("spec") {
        return Some("Tests");
    }
    if normalized.contains("doc") || normalized.ends_with(".md") {
        return Some("Docs");
    }
    if normalized.contains("docker")
        || normalized.contains("vite")
        || normalized.contains("webpack")
        || normalized.contains("cargo")
        || normalized.contains("package")
        || normalized.contains("tsconfig")
        || normalized.contains("tsup")
    {
        return Some("Tooling");
    }
    if normalized.contains("db")
        || normalized.contains("database")
        || normalized.contains("schema")
        || normalized.contains("migration")
        || normalized.contains("repository")
        || normalized.contains("persistence")
    {
        return Some("Infrastructure");
    }
    if normalized.contains("api")
        || normalized.contains("route")
        || normalized.contains("controller")
        || normalized.contains("handler")
        || normalized.contains("server")
    {
        return Some("API");
    }
    if normalized.contains("domain")
        || normalized.contains("model")
        || normalized.contains("entity")
        || normalized.contains("service")
        || normalized.contains("usecase")
        || normalized.contains("use_case")
    {
        return Some("Domain");
    }
    if normalized.contains("ui")
        || normalized.contains("component")
        || normalized.contains("page")
        || normalized.contains("view")
        || normalized.contains("screen")
        || normalized.contains("layout")
    {
        return Some("UI");
    }
    if node_type == "Module" || node_type == "File" {
        return Some("Source");
    }
    if is_symbol_type(node_type) {
        return Some("Symbols");
    }
    None
}

fn is_symbol_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "Function" | "Method" | "Class" | "Struct" | "Interface" | "Trait" | "Enum" | "Type"
    )
}

fn percentile_score(value: usize, min_for_full: usize, max_for_full: usize, max_points: u8) -> u8 {
    if value == 0 {
        return 0;
    }
    if value >= max_for_full {
        return max_points;
    }
    if value <= min_for_full {
        return ((value as f32 / min_for_full as f32) * (max_points as f32 * 0.5)).round() as u8;
    }
    let ratio = (value - min_for_full) as f32 / (max_for_full - min_for_full) as f32;
    ((max_points as f32 * 0.5) + ratio * max_points as f32 * 0.5).round() as u8
}

fn clamp_u8(value: u8, min: u8, max: u8) -> u8 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::DocumentationService;
    use devatlas_common::{
        GraphEdge, GraphNode, KnowledgeGraph, RepositoryFile, RepositoryId, ScanId, ScanResult,
        ScanStatus,
    };

    #[test]
    fn generates_semantic_documents() {
        let scan = ScanResult {
            scan_id: ScanId("scan-1".to_string()),
            repository_id: RepositoryId("repo-1".to_string()),
            status: ScanStatus::Completed,
            files_count: 3,
            folders_count: 2,
            technologies: Vec::new(),
            files: vec![
                RepositoryFile {
                    path: "src/api/user-controller.ts".to_string(),
                    extension: Some("ts".to_string()),
                    size_bytes: 10,
                },
                RepositoryFile {
                    path: "src/domain/user-service.ts".to_string(),
                    extension: Some("ts".to_string()),
                    size_bytes: 20,
                },
                RepositoryFile {
                    path: "src/db/schema.sql".to_string(),
                    extension: Some("sql".to_string()),
                    size_bytes: 30,
                },
            ],
            duration_ms: 1,
        };
        let docs = DocumentationService::generate_documents(
            &scan,
            &KnowledgeGraph {
                nodes: vec![
                    GraphNode {
                        id: "module-1".to_string(),
                        node_type: "Module".to_string(),
                        name: "src/api".to_string(),
                    },
                    GraphNode {
                        id: "file-1".to_string(),
                        node_type: "File".to_string(),
                        name: "src/api/user-controller.ts".to_string(),
                    },
                    GraphNode {
                        id: "symbol-1".to_string(),
                        node_type: "Class".to_string(),
                        name: "UserController".to_string(),
                    },
                ],
                edges: vec![GraphEdge {
                    id: "edge-1".to_string(),
                    source: "file-1".to_string(),
                    target: "symbol-1".to_string(),
                    edge_type: "Contains".to_string(),
                }],
            },
        );
        assert_eq!(docs.len(), 7);
        assert!(docs
            .iter()
            .any(|document| document.path == "docs/developer-guide.md"));
        assert!(docs
            .iter()
            .any(|document| document.path == "docs/api-summary.md"));
        assert!(docs
            .iter()
            .any(|document| document.path == "docs/database-summary.md"));
        let api = docs
            .iter()
            .find(|document| document.path == "docs/api-summary.md")
            .expect("API document should exist");
        assert!(api.content.contains("src/api/user-controller.ts"));
        assert!(api.semantic_plan.intent.contains("API candidates"));
        assert!(api.quality.source_count == 3);
    }
}
