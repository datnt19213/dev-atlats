use devatlas_app_core::AppService;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn completes_mvp_repository_knowledge_workflow() {
    let fixture_dir = unique_temp_dir("devatlas-mvp-fixture");
    let export_dir = unique_temp_dir("devatlas-mvp-export");
    create_fixture_repository(&fixture_dir);

    let repository =
        AppService::open_repository(fixture_dir.to_string_lossy().to_string()).unwrap();
    let scan = AppService::scan_repository(&repository).unwrap();
    assert!(scan.files_count >= 4);
    assert!(scan
        .technologies
        .iter()
        .any(|technology| technology.name == "TypeScript"));
    assert!(scan
        .technologies
        .iter()
        .any(|technology| technology.name == "React"));
    assert!(scan
        .technologies
        .iter()
        .any(|technology| technology.name == "SQLite"));

    let graph = AppService::build_graph_for_repository(&repository, &scan).unwrap();
    assert!(graph.nodes.iter().any(|node| node.name == "AppService"));
    assert!(graph.nodes.iter().any(|node| node.name == "GoRepository"));
    assert!(graph.edges.iter().any(|edge| edge.edge_type == "Imports"));
    assert!(graph.edges.iter().any(|edge| edge.edge_type == "Calls"));

    let documents = AppService::generate_docs(&scan, &graph);
    assert_eq!(documents.len(), 7);
    assert!(documents
        .iter()
        .any(|document| document.path == "docs/api-summary.md"));
    assert!(documents
        .iter()
        .any(|document| document.path == "docs/onboarding.md"));
    assert!(documents
        .iter()
        .any(|document| document.path == "docs/ai-context.md"));
    let developer_guide = documents
        .iter()
        .find(|document| document.path == "docs/developer-guide.md")
        .unwrap();
    assert!(developer_guide.content.contains("AppService"));
    assert!(developer_guide.content.contains("GoRepository"));
    let api = documents
        .iter()
        .find(|document| document.path == "docs/api-summary.md")
        .unwrap();
    assert!(api.content.contains("routeHandler"));
    let onboarding = documents
        .iter()
        .find(|document| document.path == "docs/onboarding.md")
        .unwrap();
    assert!(onboarding.content.contains("Suggested Workflow"));
    let ai_context = documents
        .iter()
        .find(|document| document.path == "docs/ai-context.md")
        .unwrap();
    assert!(ai_context.content.contains("Context Rules"));

    let diagrams = AppService::generate_diagrams(&scan, &graph);
    assert_eq!(diagrams.len(), 7);
    assert!(diagrams
        .iter()
        .any(|diagram| diagram.path == "diagrams/class.puml"));
    assert!(diagrams
        .iter()
        .any(|diagram| diagram.path == "diagrams/erd.mmd"));
    assert!(diagrams
        .iter()
        .any(|diagram| diagram.path == "diagrams/package.mmd"));
    assert!(diagrams
        .iter()
        .any(|diagram| diagram.path == "diagrams/architecture-overview.mmd"));
    let component = diagrams
        .iter()
        .find(|diagram| diagram.path == "diagrams/component.mmd")
        .unwrap();
    assert!(component.content.contains("Directory: src/api"));
    let dependency = diagrams
        .iter()
        .find(|diagram| diagram.path == "diagrams/dependency.puml")
        .unwrap();
    assert!(dependency.content.contains("src/api/app.ts"));
    assert!(dependency.content.contains("src/helper.ts"));
    assert!(dependency.content.contains("helper"));

    let package = AppService::export_package(&export_dir, &scan, &documents, &diagrams).unwrap();
    assert!(Path::new(&package.path).exists());
    assert!(export_dir.join("docs/README.md").exists());
    assert!(export_dir.join("docs/onboarding.md").exists());
    assert!(export_dir.join("docs/ai-context.md").exists());
    assert!(export_dir.join("diagrams/component.mmd").exists());
    assert!(export_dir.join("diagrams/class.puml").exists());
    assert!(export_dir.join("diagrams/package.mmd").exists());
    assert!(export_dir
        .join("diagrams/architecture-overview.mmd")
        .exists());
    assert!(export_dir.join("diagrams/erd.mmd").exists());
    assert!(export_dir.join("repository-summary.json").exists());
    assert!(export_dir.join("export-manifest.json").exists());
    let manifest = fs::read_to_string(export_dir.join("export-manifest.json")).unwrap();
    assert!(manifest.contains("\"packageName\": \"project-knowledge\""));

    let storage = AppService::open_memory_storage().unwrap();
    storage.save_repository(&repository).unwrap();
    storage.save_scan(&scan).unwrap();
    storage.save_export(&repository.id.0, &package).unwrap();
    assert_eq!(storage.list_repositories().unwrap().len(), 1);
    assert!(!storage
        .list_technologies(&repository.id.0)
        .unwrap()
        .is_empty());

    fs::remove_dir_all(&fixture_dir).unwrap();
    fs::remove_dir_all(&export_dir).unwrap();
}

fn create_fixture_repository(root: &Path) {
    fs::create_dir_all(root.join("src/api")).unwrap();
    fs::create_dir_all(root.join("prisma")).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"name":"fixture","dependencies":{"react":"19.0.0"}}"#,
    )
    .unwrap();
    fs::write(root.join("yarn.lock"), "# fixture").unwrap();
    fs::write(root.join("vite.config.ts"), "export default {};").unwrap();
    fs::write(
        root.join("src/api/app.ts"),
        "import { helper } from '../helper';\nexport class AppService {}\nexport function routeHandler() { return helper(); }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/helper.ts"),
        "export function helper() { return true; }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/repository.go"),
        "package src\n\nimport \"database/sql\"\n\ntype GoRepository struct {}\n",
    )
    .unwrap();
    fs::write(root.join("src/invalid.ts"), [0xff, 0xfe, 0xfd]).unwrap();
    fs::write(
        root.join("prisma/schema.prisma"),
        "datasource db {\n  provider = \"sqlite\"\n  url = \"file:dev.db\"\n}\nmodel User {\n  id Int @id\n}\n",
    )
    .unwrap();
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{timestamp}"))
}
