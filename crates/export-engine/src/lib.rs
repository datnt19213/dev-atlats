use devatlas_common::{
    now_ms, stable_id, DevAtlasError, DevAtlasResult, DiagramResult, ExportId, ExportPackage,
    GeneratedDocument, ScanResult,
};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct ExportService;

impl ExportService {
    pub fn build_knowledge_package(
        output_dir: &Path,
        scan: &ScanResult,
        documents: &[GeneratedDocument],
        diagrams: &[DiagramResult],
    ) -> DevAtlasResult<ExportPackage> {
        fs::create_dir_all(output_dir).map_err(|error| {
            DevAtlasError::new(
                "export.output_dir_failed",
                format!(
                    "Cannot create export directory {}: {error}",
                    output_dir.display()
                ),
            )
        })?;

        let mut entries = Vec::new();
        for document in documents {
            entries.push((document.path.clone(), document.content.as_bytes().to_vec()));
        }
        for diagram in diagrams {
            entries.push((diagram.path.clone(), diagram.content.as_bytes().to_vec()));
        }
        entries.push((
            "repository-summary.json".to_string(),
            summary_json(scan).into_bytes(),
        ));
        entries.push((
            "export-manifest.json".to_string(),
            manifest_json(scan, documents, diagrams, &entries)?.into_bytes(),
        ));

        materialize_artifacts(output_dir, &entries)?;

        let path = output_dir.join("project-knowledge.zip");
        write_store_zip(&path, &entries)?;
        Ok(ExportPackage {
            id: ExportId(stable_id(
                "export",
                &format!("{}-{}", scan.repository_id.0, now_ms()),
            )),
            path: path.to_string_lossy().to_string(),
            artifacts_dir: output_dir.to_string_lossy().to_string(),
            artifact_count: entries.len(),
        })
    }
}

fn materialize_artifacts(output_dir: &Path, entries: &[(String, Vec<u8>)]) -> DevAtlasResult<()> {
    for (relative_path, data) in entries {
        let artifact_path = safe_artifact_path(output_dir, relative_path)?;
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                DevAtlasError::new(
                    "export.artifact_dir_failed",
                    format!(
                        "Cannot create artifact directory {}: {error}",
                        parent.display()
                    ),
                )
            })?;
        }
        fs::write(&artifact_path, data).map_err(|error| {
            DevAtlasError::new(
                "export.artifact_write_failed",
                format!("Cannot write artifact {}: {error}", artifact_path.display()),
            )
        })?;
    }
    Ok(())
}

fn safe_artifact_path(output_dir: &Path, relative_path: &str) -> DevAtlasResult<PathBuf> {
    let normalized = normalize_zip_path(relative_path);
    if normalized.starts_with('/') || normalized.contains("..") {
        return Err(DevAtlasError::new(
            "export.invalid_artifact_path",
            format!("Invalid artifact path: {relative_path}"),
        ));
    }
    Ok(output_dir.join(normalized))
}

fn summary_json(scan: &ScanResult) -> String {
    let technologies = scan
        .technologies
        .iter()
        .map(|technology| {
            format!(
                "{{\"category\":\"{}\",\"name\":\"{}\",\"version\":{}}}",
                escape_json(technology.category.as_str()),
                escape_json(&technology.name),
                technology
                    .version
                    .as_ref()
                    .map(|version| format!("\"{}\"", escape_json(version)))
                    .unwrap_or_else(|| "null".to_string())
            )
        })
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\"repositoryId\":\"{}\",\"scanId\":\"{}\",\"files\":{},\"folders\":{},\"durationMs\":{},\"technologies\":[{}]}}",
        escape_json(&scan.repository_id.0),
        escape_json(&scan.scan_id.0),
        scan.files_count,
        scan.folders_count,
        scan.duration_ms,
        technologies
    )
}

fn manifest_json(
    scan: &ScanResult,
    documents: &[GeneratedDocument],
    diagrams: &[DiagramResult],
    entries: &[(String, Vec<u8>)],
) -> DevAtlasResult<String> {
    let artifacts = entries
        .iter()
        .map(|(path, data)| {
            json!({
                "path": path,
                "sizeBytes": data.len(),
                "category": artifact_category(path),
            })
        })
        .collect::<Vec<serde_json::Value>>();
    let value = json!({
        "schemaVersion": "1.0",
        "packageName": "project-knowledge",
        "archiveName": "project-knowledge.zip",
        "repositoryId": &scan.repository_id.0,
        "scanId": &scan.scan_id.0,
        "files": scan.files_count,
        "folders": scan.folders_count,
        "documentCount": documents.len(),
        "diagramCount": diagrams.len(),
        "enabledCategories": ["Documentation", "Diagrams", "Repository Summary", "Knowledge Package"],
        "supportedFormats": ["Markdown", "Mermaid", "PlantUML", "SVG", "JSON", "ZIP"],
        "futureFormatsDisabled": ["HTML", "PDF", "DOCX", "ODT", "RTF", "PNG", "JPEG", "WebP", "TAR", "AI Context Provider Packages"],
        "artifacts": artifacts,
    });
    serde_json::to_string_pretty(&value).map_err(json_error)
}

fn artifact_category(path: &str) -> &'static str {
    if path.starts_with("docs/") {
        "Documentation"
    } else if path.starts_with("diagrams/") {
        "Diagram"
    } else if path == "repository-summary.json" {
        "Repository Summary"
    } else {
        "Metadata"
    }
}

fn write_store_zip(path: &Path, entries: &[(String, Vec<u8>)]) -> DevAtlasResult<()> {
    let mut file = fs::File::create(path).map_err(|error| {
        DevAtlasError::new(
            "export.zip_create_failed",
            format!("{}: {error}", path.display()),
        )
    })?;
    let mut central_directory = Vec::new();
    let mut offset = 0_u32;

    for (name, data) in entries {
        let name_bytes = normalize_zip_path(name).into_bytes();
        let crc = crc32(data);
        let size = u32::try_from(data.len()).unwrap_or(u32::MAX);
        let local = local_header(&name_bytes, crc, size);
        file.write_all(&local).map_err(io_error)?;
        file.write_all(&name_bytes).map_err(io_error)?;
        file.write_all(data).map_err(io_error)?;
        central_directory.extend(central_header(&name_bytes, crc, size, offset));
        offset = offset
            .saturating_add(u32::try_from(local.len()).unwrap_or(0))
            .saturating_add(u32::try_from(name_bytes.len()).unwrap_or(0))
            .saturating_add(size);
    }

    let central_offset = offset;
    file.write_all(&central_directory).map_err(io_error)?;
    let central_size = u32::try_from(central_directory.len()).unwrap_or(u32::MAX);
    file.write_all(&end_record(
        entries.len() as u16,
        central_size,
        central_offset,
    ))
    .map_err(io_error)?;
    Ok(())
}

fn local_header(name: &[u8], crc: u32, size: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend([0x50, 0x4b, 0x03, 0x04]);
    bytes.extend(20_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(crc.to_le_bytes());
    bytes.extend(size.to_le_bytes());
    bytes.extend(size.to_le_bytes());
    bytes.extend((name.len() as u16).to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes
}

fn central_header(name: &[u8], crc: u32, size: u32, offset: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend([0x50, 0x4b, 0x01, 0x02]);
    bytes.extend(20_u16.to_le_bytes());
    bytes.extend(20_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(crc.to_le_bytes());
    bytes.extend(size.to_le_bytes());
    bytes.extend(size.to_le_bytes());
    bytes.extend((name.len() as u16).to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(offset.to_le_bytes());
    bytes.extend(name);
    bytes
}

fn end_record(entry_count: u16, central_size: u32, central_offset: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend([0x50, 0x4b, 0x05, 0x06]);
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(entry_count.to_le_bytes());
    bytes.extend(entry_count.to_le_bytes());
    bytes.extend(central_size.to_le_bytes());
    bytes.extend(central_offset.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn normalize_zip_path(path: &str) -> String {
    PathBuf::from(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join("/")
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn io_error(error: std::io::Error) -> DevAtlasError {
    DevAtlasError::new("export.zip_write_failed", error.to_string())
}

fn json_error(error: serde_json::Error) -> DevAtlasError {
    DevAtlasError::new("export.json_failed", error.to_string())
}

#[cfg(test)]
mod tests {
    use super::ExportService;
    use devatlas_common::{RepositoryId, ScanId, ScanResult, ScanStatus};

    #[test]
    fn creates_zip_package() {
        let output_dir = std::env::temp_dir().join("devatlas-export-test");
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
        let package = ExportService::build_knowledge_package(&output_dir, &scan, &[], &[]).unwrap();
        assert!(package.path.ends_with("project-knowledge.zip"));
        assert!(std::path::Path::new(&package.path).exists());
        assert!(std::path::Path::new(&package.artifacts_dir)
            .join("repository-summary.json")
            .exists());
        let manifest_path =
            std::path::Path::new(&package.artifacts_dir).join("export-manifest.json");
        assert!(manifest_path.exists());
        let manifest = std::fs::read_to_string(manifest_path).unwrap();
        assert!(manifest.contains("\"schemaVersion\": \"1.0\""));
        assert!(manifest.contains("\"archiveName\": \"project-knowledge.zip\""));
        assert_eq!(package.artifact_count, 2);
    }
}
