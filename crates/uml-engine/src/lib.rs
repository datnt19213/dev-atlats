use devatlas_common::{
    stable_id, DiagramFormat, DiagramId, DiagramResult, DiagramType, KnowledgeGraph, ScanResult,
};
use std::collections::{BTreeSet, HashMap};

pub struct UmlService;

impl UmlService {
    pub fn generate_diagrams(scan: &ScanResult, graph: &KnowledgeGraph) -> Vec<DiagramResult> {
        vec![
            DiagramResult {
                id: DiagramId(stable_id("diagram", "class")),
                path: "diagrams/class.puml".to_string(),
                diagram_type: DiagramType::Class,
                format: DiagramFormat::PlantUml,
                content: class_plantuml(graph),
            },
            DiagramResult {
                id: DiagramId(stable_id("diagram", "component")),
                path: "diagrams/component.mmd".to_string(),
                diagram_type: DiagramType::Component,
                format: DiagramFormat::Mermaid,
                content: component_mermaid(scan, graph),
            },
            DiagramResult {
                id: DiagramId(stable_id("diagram", "dependency")),
                path: "diagrams/dependency.puml".to_string(),
                diagram_type: DiagramType::Dependency,
                format: DiagramFormat::PlantUml,
                content: dependency_plantuml(graph),
            },
            DiagramResult {
                id: DiagramId(stable_id("diagram", "erd")),
                path: "diagrams/erd.mmd".to_string(),
                diagram_type: DiagramType::Erd,
                format: DiagramFormat::Mermaid,
                content: erd_mermaid(scan),
            },
            DiagramResult {
                id: DiagramId(stable_id("diagram", "folder")),
                path: "diagrams/folder-structure.svg".to_string(),
                diagram_type: DiagramType::FolderStructure,
                format: DiagramFormat::Svg,
                content: folder_svg(scan),
            },
            DiagramResult {
                id: DiagramId(stable_id("diagram", "package")),
                path: "diagrams/package.mmd".to_string(),
                diagram_type: DiagramType::Package,
                format: DiagramFormat::Mermaid,
                content: package_mermaid(scan, graph),
            },
            DiagramResult {
                id: DiagramId(stable_id("diagram", "architecture-overview")),
                path: "diagrams/architecture-overview.mmd".to_string(),
                diagram_type: DiagramType::ArchitectureOverview,
                format: DiagramFormat::Mermaid,
                content: architecture_overview_mermaid(scan, graph),
            },
        ]
    }
}

fn class_plantuml(graph: &KnowledgeGraph) -> String {
    let symbol_lines = graph
        .nodes
        .iter()
        .filter(|node| is_class_diagram_symbol(&node.node_type))
        .take(100)
        .map(|node| {
            let keyword = match node.node_type.as_str() {
                "Interface" | "Trait" => "interface",
                _ => "class",
            };
            format!(
                "{keyword} \"{}\" <<{}>>",
                escape_diagram_label(&node.name),
                escape_diagram_label(&node.node_type)
            )
        })
        .collect::<Vec<String>>();
    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<&str, &devatlas_common::GraphNode>>();
    let ownership_lines = graph
        .edges
        .iter()
        .filter(|edge| edge.edge_type == "Contains" || edge.edge_type == "Owns")
        .filter_map(|edge| {
            let source = nodes_by_id.get(edge.source.as_str())?;
            let target = nodes_by_id.get(edge.target.as_str())?;
            if edge.edge_type == "Owns" && is_class_diagram_symbol(&target.node_type) {
                return Some(format!(
                    "\"{}\" *-- \"{}\" : owns",
                    escape_diagram_label(&source.name),
                    escape_diagram_label(&target.name)
                ));
            }
            if source.node_type == "File" && is_class_diagram_symbol(&target.node_type) {
                Some(format!(
                    "\"{}\" ..> \"{}\" : contains",
                    escape_diagram_label(&source.name),
                    escape_diagram_label(&target.name)
                ))
            } else {
                None
            }
        })
        .take(100)
        .collect::<Vec<String>>();

    let mut lines = vec!["@startuml".to_string()];
    if symbol_lines.is_empty() {
        lines.push("class \"No class-like symbols detected\" as NoSymbols".to_string());
    } else {
        lines.extend(symbol_lines);
        lines.extend(ownership_lines);
    }
    lines.push("@enduml".to_string());
    lines.push(String::new());
    lines.join("\n")
}

fn component_mermaid(scan: &ScanResult, graph: &KnowledgeGraph) -> String {
    let mut lines = vec![
        "flowchart TD".to_string(),
        "    Repository[\"Repository\"]".to_string(),
    ];
    for (index, technology) in scan.technologies.iter().take(50).enumerate() {
        let node_id = format!("Technology_{index}");
        lines.push(format!(
            "    {node_id}[\"{}: {}\"]",
            escape_diagram_label(technology.category.as_str()),
            escape_diagram_label(&technology.name)
        ));
        lines.push(format!("    Repository --> {node_id}"));
    }
    for (index, module) in graph
        .nodes
        .iter()
        .filter(|node| node.node_type == "Directory")
        .take(100)
        .enumerate()
    {
        let node_id = format!("Module_{index}");
        lines.push(format!(
            "    {node_id}[\"Directory: {}\"]",
            escape_diagram_label(&module.name)
        ));
        lines.push(format!("    Repository --> {node_id}"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn dependency_plantuml(graph: &KnowledgeGraph) -> String {
    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.name.as_str()))
        .collect::<HashMap<&str, &str>>();
    let edges = graph
        .edges
        .iter()
        .filter(|edge| matches!(edge.edge_type.as_str(), "Imports" | "Uses" | "Calls"))
        .take(250)
        .map(|edge| {
            let source = nodes_by_id
                .get(edge.source.as_str())
                .copied()
                .unwrap_or(edge.source.as_str());
            let target = nodes_by_id
                .get(edge.target.as_str())
                .copied()
                .unwrap_or(edge.target.as_str());
            format!(
                "\"{}\" --> \"{}\" : {}",
                escape_diagram_label(source),
                escape_diagram_label(target),
                escape_diagram_label(&edge.edge_type)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    format!("@startuml\n{}\n@enduml\n", edges)
}

fn erd_mermaid(scan: &ScanResult) -> String {
    let database_nodes = scan
        .technologies
        .iter()
        .filter(|technology| {
            let category = technology.category.as_str();
            category == "Database" || category == "ORM"
        })
        .map(|technology| sanitize_entity(&technology.name))
        .collect::<Vec<String>>();
    let schema_nodes = scan
        .files
        .iter()
        .filter(|file| is_schema_file(&file.path))
        .take(50)
        .map(|file| sanitize_entity(&file.path))
        .collect::<Vec<String>>();

    if database_nodes.is_empty() && schema_nodes.is_empty() {
        return "erDiagram\n    REPOSITORY_SCHEMA {\n        string status \"No database schema detected\"\n    }\n"
            .to_string();
    }

    let mut lines = vec!["erDiagram".to_string()];
    for database in &database_nodes {
        lines.push(format!(
            "    {database} {{\n        string source \"Detected technology\"\n    }}"
        ));
    }
    for schema in &schema_nodes {
        lines.push(format!(
            "    {schema} {{\n        string path \"Candidate schema file\"\n    }}"
        ));
    }
    for database in &database_nodes {
        for schema in &schema_nodes {
            lines.push(format!(
                "    {database} ||--o{{ {schema} : \"contains candidate schema\""
            ));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn folder_svg(scan: &ScanResult) -> String {
    let mut folders = scan
        .files
        .iter()
        .filter_map(|file| {
            file.path
                .rsplit_once('/')
                .map(|(directory, _)| directory.to_string())
        })
        .collect::<BTreeSet<String>>();
    if folders.is_empty() {
        folders.insert(".".to_string());
    }
    let rows = folders
        .iter()
        .take(120)
        .enumerate()
        .map(|(index, folder)| {
            let y = 30 + (index * 22);
            let depth = folder.matches('/').count();
            let x = 16 + depth.min(8) * 18;
            format!(
                "<text x=\"{}\" y=\"{}\" font-size=\"14\">{}</text>",
                x,
                y,
                escape_xml(folder)
            )
        })
        .collect::<Vec<String>>()
        .join("");
    let height = (folders.len().min(120) * 22 + 48).max(180);
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"980\" height=\"{}\" viewBox=\"0 0 980 {}\"><rect width=\"980\" height=\"{}\" fill=\"#ffffff\"/><text x=\"16\" y=\"18\" font-size=\"16\" font-weight=\"700\">Folder Structure: {} folders, {} files</text>{}</svg>",
        height,
        height,
        height,
        scan.folders_count,
        scan.files_count,
        rows
    )
}

fn package_mermaid(scan: &ScanResult, graph: &KnowledgeGraph) -> String {
    let mut package_names = scan
        .files
        .iter()
        .filter_map(|file| package_name_from_path(&file.path))
        .take(100)
        .collect::<Vec<String>>();
    package_names.sort();
    package_names.dedup();

    let mut lines = vec!["flowchart TD".to_string()];
    if package_names.is_empty() {
        lines.push("    Repository[\"No packages detected\"]".to_string());
        lines.push(String::new());
        return lines.join("\n");
    }

    let package_ids = package_names
        .iter()
        .enumerate()
        .map(|(index, package)| (package.as_str(), format!("Package_{index}")))
        .collect::<HashMap<&str, String>>();
    for package in &package_names {
        if let Some(node_id) = package_ids.get(package.as_str()) {
            lines.push(format!(
                "    {node_id}[\"Package: {}\"]",
                escape_diagram_label(package)
            ));
        }
    }

    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.name.as_str()))
        .collect::<HashMap<&str, &str>>();
    let mut package_edges = graph
        .edges
        .iter()
        .filter(|edge| matches!(edge.edge_type.as_str(), "Imports" | "Uses" | "Calls"))
        .filter_map(|edge| {
            let source = nodes_by_id.get(edge.source.as_str()).copied()?;
            let source_package = package_name_from_path(source)?;
            let target = nodes_by_id
                .get(edge.target.as_str())
                .copied()
                .unwrap_or_default();
            let target_package =
                package_name_from_path(target).unwrap_or_else(|| target.to_string());
            if source_package == target_package {
                None
            } else {
                Some((source_package, target_package))
            }
        })
        .collect::<Vec<(String, String)>>();
    package_edges.sort();
    package_edges.dedup();

    for (index, (source, target)) in package_edges.iter().take(100).enumerate() {
        let source_id = package_ids
            .get(source.as_str())
            .cloned()
            .unwrap_or_else(|| format!("ExternalSource_{index}"));
        let target_id = package_ids
            .get(target.as_str())
            .cloned()
            .unwrap_or_else(|| format!("ExternalTarget_{index}"));
        if !package_ids.contains_key(target.as_str()) {
            lines.push(format!(
                "    {target_id}[\"External: {}\"]",
                escape_diagram_label(target)
            ));
        }
        lines.push(format!("    {source_id} --> {target_id}"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn architecture_overview_mermaid(scan: &ScanResult, graph: &KnowledgeGraph) -> String {
    let mut lines = vec![
        "flowchart LR".to_string(),
        "    Repository[\"Repository\"]".to_string(),
        format!("    Scanner[\"Scanner: {} files\"]", scan.files_count),
        format!(
            "    Graph[\"Knowledge Graph: {} nodes\"]",
            graph.nodes.len()
        ),
        "    Documentation[\"Documentation\"]".to_string(),
        "    Diagrams[\"Diagrams\"]".to_string(),
        "    Export[\"Knowledge Package\"]".to_string(),
        "    Repository --> Scanner".to_string(),
        "    Scanner --> Graph".to_string(),
        "    Graph --> Documentation".to_string(),
        "    Graph --> Diagrams".to_string(),
        "    Documentation --> Export".to_string(),
        "    Diagrams --> Export".to_string(),
    ];
    for (index, technology) in scan.technologies.iter().take(50).enumerate() {
        let node_id = format!("Technology_{index}");
        lines.push(format!(
            "    {node_id}[\"{}: {}\"]",
            escape_diagram_label(technology.category.as_str()),
            escape_diagram_label(&technology.name)
        ));
        lines.push(format!("    Scanner --> {node_id}"));
    }
    let mut package_names = scan
        .files
        .iter()
        .filter_map(|file| package_name_from_path(&file.path))
        .collect::<Vec<String>>();
    package_names.sort();
    package_names.dedup();
    for (index, package) in package_names.iter().take(24).enumerate() {
        let node_id = format!("Package_{index}");
        lines.push(format!(
            "    {node_id}[\"Package: {}\"]",
            escape_diagram_label(package)
        ));
        lines.push(format!("    Graph --> {node_id}"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn escape_diagram_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn sanitize_entity(name: &str) -> String {
    let entity = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if entity.is_empty() {
        "UNKNOWN_SCHEMA".to_string()
    } else {
        entity
    }
}

fn is_class_diagram_symbol(node_type: &str) -> bool {
    matches!(
        node_type,
        "Class" | "Interface" | "Struct" | "Trait" | "Method"
    )
}

fn package_name_from_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let package = normalized
        .rsplit_once('/')
        .map(|(directory, _file)| directory)
        .unwrap_or(normalized.as_str())
        .trim_matches('/');
    if package.is_empty() || package == normalized {
        None
    } else {
        Some(package.to_string())
    }
}

fn is_schema_file(path: &str) -> bool {
    let lower_path = path.to_lowercase();
    lower_path.contains("schema")
        || lower_path.contains("migration")
        || lower_path.ends_with(".prisma")
        || lower_path.ends_with(".sql")
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::UmlService;
    use devatlas_common::{
        DiagramType, GraphEdge, GraphNode, KnowledgeGraph, RepositoryFile, RepositoryId, ScanId,
        ScanResult, ScanStatus, Technology, TechnologyCategory,
    };

    #[test]
    fn generates_diagram_outputs() {
        let scan = ScanResult {
            scan_id: ScanId("scan-1".to_string()),
            repository_id: RepositoryId("repo-1".to_string()),
            status: ScanStatus::Completed,
            files_count: 0,
            folders_count: 0,
            technologies: vec![Technology {
                category: TechnologyCategory::Orm,
                name: "Prisma".to_string(),
                version: None,
            }],
            files: vec![RepositoryFile {
                path: "prisma/schema.prisma".to_string(),
                extension: Some("prisma".to_string()),
                size_bytes: 1,
            }],
            duration_ms: 0,
        };
        let diagrams = UmlService::generate_diagrams(
            &scan,
            &KnowledgeGraph {
                nodes: vec![
                    GraphNode {
                        id: "module-1".to_string(),
                        node_type: "Directory".to_string(),
                        name: "src/api".to_string(),
                    },
                    GraphNode {
                        id: "file-1".to_string(),
                        node_type: "File".to_string(),
                        name: "src/api/app.ts".to_string(),
                    },
                    GraphNode {
                        id: "dependency-1".to_string(),
                        node_type: "Dependency".to_string(),
                        name: "../helper".to_string(),
                    },
                    GraphNode {
                        id: "symbol-1".to_string(),
                        node_type: "Class".to_string(),
                        name: "AppService".to_string(),
                    },
                    GraphNode {
                        id: "symbol-2".to_string(),
                        node_type: "Method".to_string(),
                        name: "refresh".to_string(),
                    },
                ],
                edges: vec![
                    GraphEdge {
                        id: "edge-1".to_string(),
                        source: "file-1".to_string(),
                        target: "dependency-1".to_string(),
                        edge_type: "Imports".to_string(),
                    },
                    GraphEdge {
                        id: "edge-2".to_string(),
                        source: "file-1".to_string(),
                        target: "symbol-1".to_string(),
                        edge_type: "Contains".to_string(),
                    },
                    GraphEdge {
                        id: "edge-3".to_string(),
                        source: "symbol-1".to_string(),
                        target: "symbol-2".to_string(),
                        edge_type: "Owns".to_string(),
                    },
                ],
            },
        );
        assert_eq!(diagrams.len(), 7);
        let class = diagrams
            .iter()
            .find(|diagram| diagram.diagram_type == DiagramType::Class)
            .expect("class diagram should be generated");
        assert!(class.content.contains("AppService"));
        assert!(class.content.contains("refresh"));
        assert!(class.content.contains("owns"));
        let component = diagrams
            .iter()
            .find(|diagram| diagram.diagram_type == DiagramType::Component)
            .expect("component diagram should be generated");
        assert!(component.content.contains("Directory: src/api"));
        let dependency = diagrams
            .iter()
            .find(|diagram| diagram.diagram_type == DiagramType::Dependency)
            .expect("dependency diagram should be generated");
        assert!(dependency.content.contains("src/api/app.ts"));
        assert!(dependency.content.contains("../helper"));
        assert!(!dependency.content.contains("dependency-1"));
        let erd = diagrams
            .iter()
            .find(|diagram| diagram.diagram_type == DiagramType::Erd)
            .expect("ERD should be generated");
        assert!(erd.content.contains("PRISMA"));
        assert!(erd.content.contains("PRISMA_SCHEMA_PRISMA"));
        let package = diagrams
            .iter()
            .find(|diagram| diagram.diagram_type == DiagramType::Package)
            .expect("package diagram should be generated");
        assert!(package.content.contains("Package: prisma"));
        let architecture = diagrams
            .iter()
            .find(|diagram| diagram.diagram_type == DiagramType::ArchitectureOverview)
            .expect("architecture overview should be generated");
        assert!(architecture.content.contains("Knowledge Package"));
    }
}
