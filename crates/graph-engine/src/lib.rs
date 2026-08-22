use devatlas_common::{stable_id, GraphEdge, GraphNode, KnowledgeGraph, ScanResult};
use devatlas_parser_engine::ParsedRepository;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::{BTreeSet, HashMap, HashSet};

pub struct GraphService;

impl GraphService {
    pub fn build_graph(scan: &ScanResult) -> KnowledgeGraph {
        Self::build_graph_with_parsed(
            scan,
            &ParsedRepository {
                modules: Vec::new(),
                symbols: Vec::new(),
                relationships: Vec::new(),
            },
        )
    }

    pub fn build_graph_with_parsed(
        scan: &ScanResult,
        parsed_repository: &ParsedRepository,
    ) -> KnowledgeGraph {
        let mut graph = Graph::<GraphNode, GraphEdge>::new();
        let repository_node_id = scan.repository_id.0.clone();
        let repository_index = graph.add_node(GraphNode {
            id: repository_node_id.clone(),
            node_type: "Repository".to_string(),
            name: repository_node_id.clone(),
        });
        let mut node_indexes = HashMap::from([(repository_node_id.clone(), repository_index)]);
        let file_paths = scan
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<HashSet<&str>>();
        let symbol_ids = parsed_repository
            .symbols
            .iter()
            .map(|symbol| symbol.id.as_str())
            .collect::<HashSet<&str>>();

        for technology in &scan.technologies {
            let node_id = stable_id("technology", &technology.name);
            let technology_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: node_id.clone(),
                    node_type: technology.category.as_str().to_string(),
                    name: technology.name.clone(),
                },
            );
            graph.add_edge(
                repository_index,
                technology_index,
                GraphEdge {
                    id: stable_id("edge", &format!("{repository_node_id}-{node_id}")),
                    source: repository_node_id.clone(),
                    target: node_id,
                    edge_type: "Uses".to_string(),
                },
            );
        }

        let directory_paths = directory_paths_from_scan(scan);
        for directory_path in &directory_paths {
            let directory_id = stable_id("directory", directory_path);
            let directory_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: directory_id.clone(),
                    node_type: "Directory".to_string(),
                    name: directory_path.clone(),
                },
            );
            let parent_id = directory_parent(directory_path)
                .map(|parent| stable_id("directory", &parent))
                .unwrap_or_else(|| repository_node_id.clone());
            let parent_index = node_indexes
                .get(&parent_id)
                .copied()
                .unwrap_or(repository_index);
            graph.add_edge(
                parent_index,
                directory_index,
                GraphEdge {
                    id: stable_id("edge", &format!("{parent_id}-{directory_id}")),
                    source: parent_id,
                    target: directory_id,
                    edge_type: "Contains".to_string(),
                },
            );
        }

        for file in &scan.files {
            let node_id = stable_id("file", &file.path);
            let file_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: node_id.clone(),
                    node_type: "File".to_string(),
                    name: file.path.clone(),
                },
            );
            let parent_id = file
                .path
                .rsplit_once('/')
                .map(|(directory, _)| stable_id("directory", directory))
                .unwrap_or_else(|| repository_node_id.clone());
            let parent_index = node_indexes
                .get(&parent_id)
                .copied()
                .unwrap_or(repository_index);
            graph.add_edge(
                parent_index,
                file_index,
                GraphEdge {
                    id: stable_id("edge", &format!("{parent_id}-{node_id}")),
                    source: parent_id,
                    target: node_id,
                    edge_type: "Contains".to_string(),
                },
            );
        }

        for module in &parsed_repository.modules {
            let node_id = stable_id("module", &module.path);
            let module_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: node_id.clone(),
                    node_type: "Module".to_string(),
                    name: module.path.clone(),
                },
            );
            graph.add_edge(
                repository_index,
                module_index,
                GraphEdge {
                    id: stable_id("edge", &format!("{repository_node_id}-{node_id}")),
                    source: repository_node_id.clone(),
                    target: node_id,
                    edge_type: "Contains".to_string(),
                },
            );
        }

        for symbol in &parsed_repository.symbols {
            let file_id = stable_id("file", &symbol.path);
            let file_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: file_id.clone(),
                    node_type: "File".to_string(),
                    name: symbol.path.clone(),
                },
            );
            let symbol_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: symbol.id.clone(),
                    node_type: symbol.symbol_type.as_str().to_string(),
                    name: symbol.name.clone(),
                },
            );
            graph.add_edge(
                file_index,
                symbol_index,
                GraphEdge {
                    id: stable_id("edge", &format!("{file_id}-{}", symbol.id)),
                    source: file_id,
                    target: symbol.id.clone(),
                    edge_type: "Contains".to_string(),
                },
            );
            if let Some(owner_id) = &symbol.owner_id {
                let Some(owner) = parsed_repository
                    .symbols
                    .iter()
                    .find(|candidate| candidate.id == *owner_id)
                else {
                    continue;
                };
                let owner_index = add_unique_node(
                    &mut graph,
                    &mut node_indexes,
                    GraphNode {
                        id: owner.id.clone(),
                        node_type: owner.symbol_type.as_str().to_string(),
                        name: owner.name.clone(),
                    },
                );
                graph.add_edge(
                    owner_index,
                    symbol_index,
                    GraphEdge {
                        id: stable_id("edge", &format!("{}-{}", owner.id, symbol.id)),
                        source: owner.id.clone(),
                        target: symbol.id.clone(),
                        edge_type: "Owns".to_string(),
                    },
                );
            }
        }

        for relationship in &parsed_repository.relationships {
            let source_id = stable_id("file", &relationship.source_path);
            let source_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: source_id.clone(),
                    node_type: "File".to_string(),
                    name: relationship.source_path.clone(),
                },
            );
            let (target_id, target_node_type, target_name) =
                if symbol_ids.contains(relationship.target.as_str()) {
                    let symbol = parsed_repository
                        .symbols
                        .iter()
                        .find(|symbol| symbol.id == relationship.target)
                        .expect("symbol id set must reference parsed symbol");
                    (
                        symbol.id.clone(),
                        symbol.symbol_type.as_str().to_string(),
                        symbol.name.clone(),
                    )
                } else if file_paths.contains(relationship.target.as_str()) {
                    (
                        stable_id("file", &relationship.target),
                        "File".to_string(),
                        relationship.target.clone(),
                    )
                } else {
                    (
                        stable_id("dependency", &relationship.target),
                        "Dependency".to_string(),
                        relationship.target.clone(),
                    )
                };
            let target_index = add_unique_node(
                &mut graph,
                &mut node_indexes,
                GraphNode {
                    id: target_id.clone(),
                    node_type: target_node_type,
                    name: target_name,
                },
            );
            graph.add_edge(
                source_index,
                target_index,
                GraphEdge {
                    id: stable_id("edge", &format!("{source_id}-{target_id}")),
                    source: source_id,
                    target: target_id,
                    edge_type: relationship.relationship_type.as_str().to_string(),
                },
            );
        }

        KnowledgeGraph {
            nodes: graph.node_weights().cloned().collect(),
            edges: graph.edge_weights().cloned().collect(),
        }
    }

    pub fn find_dependencies(graph: &KnowledgeGraph, source_id: &str) -> Vec<GraphNode> {
        let nodes_by_id = graph
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect::<HashMap<&str, &GraphNode>>();
        graph
            .edges
            .iter()
            .filter(|edge| edge.source == source_id)
            .filter_map(|edge| nodes_by_id.get(edge.target.as_str()).copied())
            .cloned()
            .collect()
    }
}

fn directory_paths_from_scan(scan: &ScanResult) -> Vec<String> {
    let mut paths = BTreeSet::new();
    for file in &scan.files {
        let Some((directory, _file_name)) = file.path.rsplit_once('/') else {
            continue;
        };
        let mut current = String::new();
        for part in directory.split('/').filter(|part| !part.is_empty()) {
            if current.is_empty() {
                current.push_str(part);
            } else {
                current.push('/');
                current.push_str(part);
            }
            paths.insert(current.clone());
        }
    }
    paths.into_iter().collect()
}

fn directory_parent(directory_path: &str) -> Option<String> {
    directory_path
        .rsplit_once('/')
        .map(|(parent, _child)| parent.to_string())
}

fn add_unique_node(
    graph: &mut Graph<GraphNode, GraphEdge>,
    node_indexes: &mut HashMap<String, NodeIndex>,
    node: GraphNode,
) -> NodeIndex {
    if let Some(index) = node_indexes.get(&node.id) {
        return *index;
    }
    let index = graph.add_node(node.clone());
    node_indexes.insert(node.id, index);
    index
}

#[cfg(test)]
mod tests {
    use super::GraphService;
    use devatlas_common::{RepositoryFile, RepositoryId, ScanId, ScanResult, ScanStatus};
    use devatlas_parser_engine::{ParsedRepository, ParsedSymbol, SymbolType};

    #[test]
    fn creates_repository_node() {
        let scan = ScanResult {
            scan_id: ScanId("scan-1".to_string()),
            repository_id: RepositoryId("repo-1".to_string()),
            status: ScanStatus::Completed,
            files_count: 0,
            folders_count: 0,
            technologies: Vec::new(),
            files: Vec::new(),
            duration_ms: 0,
        };
        let graph = GraphService::build_graph(&scan);
        assert_eq!(graph.nodes[0].node_type, "Repository");
    }

    #[test]
    fn finds_repository_dependencies() {
        let scan = ScanResult {
            scan_id: ScanId("scan-1".to_string()),
            repository_id: RepositoryId("repo-1".to_string()),
            status: ScanStatus::Completed,
            files_count: 0,
            folders_count: 0,
            technologies: vec![devatlas_common::Technology {
                category: devatlas_common::TechnologyCategory::Language,
                name: "Rust".to_string(),
                version: None,
            }],
            files: Vec::new(),
            duration_ms: 0,
        };
        let graph = GraphService::build_graph(&scan);
        let dependencies = GraphService::find_dependencies(&graph, "repo-1");
        assert!(dependencies.iter().any(|node| node.name == "Rust"));
    }

    #[test]
    fn adds_parsed_symbols_to_graph() {
        let scan = ScanResult {
            scan_id: ScanId("scan-1".to_string()),
            repository_id: RepositoryId("repo-1".to_string()),
            status: ScanStatus::Completed,
            files_count: 1,
            folders_count: 1,
            technologies: Vec::new(),
            files: vec![RepositoryFile {
                path: "src/scanner.rs".to_string(),
                extension: Some("rs".to_string()),
                size_bytes: 1,
            }],
            duration_ms: 0,
        };
        let parsed = ParsedRepository {
            modules: Vec::new(),
            symbols: vec![
                ParsedSymbol {
                    id: "symbol-1".to_string(),
                    name: "ScannerService".to_string(),
                    path: "src/scanner.rs".to_string(),
                    symbol_type: SymbolType::Struct,
                    line: 1,
                    owner_id: None,
                },
                ParsedSymbol {
                    id: "symbol-2".to_string(),
                    name: "scan".to_string(),
                    path: "src/scanner.rs".to_string(),
                    symbol_type: SymbolType::Method,
                    line: 2,
                    owner_id: Some("symbol-1".to_string()),
                },
            ],
            relationships: Vec::new(),
        };
        let graph = GraphService::build_graph_with_parsed(&scan, &parsed);
        assert!(graph.nodes.iter().any(|node| node.name == "ScannerService"));
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.node_type == "Directory" && node.name == "src"));
        assert!(graph.edges.iter().any(|edge| edge.edge_type == "Contains"));
        assert!(graph.edges.iter().any(|edge| {
            edge.source == "symbol-1" && edge.target == "symbol-2" && edge.edge_type == "Owns"
        }));
    }
}
