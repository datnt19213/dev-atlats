use devatlas_common::{
    now_ms, stable_id, DevAtlasError, DevAtlasResult, RepositoryFile, RepositoryId, RepositoryPath,
    ScanId, ScanResult, ScanStatus, Technology, TechnologyCategory,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const IGNORED_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    "coverage",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
    ".parcel-cache",
    ".svelte-kit",
    ".vercel",
    ".vite",
    "vendor",
    "__pycache__",
];

#[derive(Clone, Debug, Default)]
pub struct ScanOptions {
    pub max_files: Option<usize>,
    pub include_paths: Vec<String>,
}

pub struct ScannerService;

impl ScannerService {
    pub fn scan_repository(
        repository_id: RepositoryId,
        repository_path: &RepositoryPath,
    ) -> DevAtlasResult<ScanResult> {
        Self::scan_repository_with_options(repository_id, repository_path, &ScanOptions::default())
    }

    pub fn scan_repository_with_options(
        repository_id: RepositoryId,
        repository_path: &RepositoryPath,
        options: &ScanOptions,
    ) -> DevAtlasResult<ScanResult> {
        let started_at = now_ms();
        let mut files = Vec::new();
        let mut folders_count = 0_usize;
        collect_files(
            repository_path.as_path(),
            repository_path.as_path(),
            &mut files,
            &mut folders_count,
            options,
        )?;
        let technologies = detect_technologies(repository_path.as_path(), &files);
        let duration_ms = now_ms().saturating_sub(started_at);
        Ok(ScanResult {
            scan_id: ScanId(stable_id(
                "scan",
                &format!("{}-{started_at}", repository_id.0),
            )),
            repository_id,
            status: ScanStatus::Completed,
            files_count: files.len(),
            folders_count,
            technologies,
            files,
            duration_ms,
        })
    }
}

fn path_matches_scope(relative_path: &str, include_paths: &[String]) -> bool {
    if include_paths.is_empty() {
        return true;
    }
    include_paths.iter().any(|include_path| {
        let normalized = normalize_path(Path::new(include_path));
        relative_path == normalized || relative_path.starts_with(&format!("{normalized}/"))
    })
}

fn collect_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<RepositoryFile>,
    folders_count: &mut usize,
    options: &ScanOptions,
) -> DevAtlasResult<()> {
    if options
        .max_files
        .is_some_and(|max_files| files.len() >= max_files)
    {
        return Ok(());
    }

    let entries = fs::read_dir(current).map_err(|error| {
        DevAtlasError::new(
            "scanner.read_dir_failed",
            format!("Cannot read directory {}: {error}", current.display()),
        )
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|error| DevAtlasError::new("scanner.entry_failed", error.to_string()))?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if IGNORED_DIRS.contains(&file_name.as_str()) {
                continue;
            }
            *folders_count += 1;
            collect_files(root, &path, files, folders_count, options)?;
            continue;
        }
        if path.is_file() {
            if options
                .max_files
                .is_some_and(|max_files| files.len() >= max_files)
            {
                break;
            }
            let metadata = entry.metadata().map_err(|error| {
                DevAtlasError::new(
                    "scanner.metadata_failed",
                    format!("{}: {error}", path.display()),
                )
            })?;
            let relative_path = normalize_path(path.strip_prefix(root).unwrap_or(&path));
            if !path_matches_scope(&relative_path, &options.include_paths) {
                continue;
            }
            let extension = path
                .extension()
                .map(|value| value.to_string_lossy().to_string());
            files.push(RepositoryFile {
                path: relative_path,
                extension,
                size_bytes: metadata.len(),
            });
        }
    }
    Ok(())
}

pub fn detect_technologies(root: &Path, files: &[RepositoryFile]) -> Vec<Technology> {
    let mut technologies = Vec::new();
    let names: BTreeSet<String> = files.iter().map(|file| file.path.to_lowercase()).collect();

    add_languages(files, &mut technologies);
    add_if(
        &names,
        "package.json",
        TechnologyCategory::PackageManager,
        "npm",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "yarn.lock",
        TechnologyCategory::PackageManager,
        "Yarn",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "pnpm-lock.yaml",
        TechnologyCategory::PackageManager,
        "pnpm",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "cargo.toml",
        TechnologyCategory::Language,
        "Rust",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "tauri.conf.json",
        TechnologyCategory::Framework,
        "Tauri",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "src-tauri/tauri.conf.json",
        TechnologyCategory::Framework,
        "Tauri",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "vite.config.ts",
        TechnologyCategory::Framework,
        "Vite",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "vite.config.js",
        TechnologyCategory::Framework,
        "Vite",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "next.config.js",
        TechnologyCategory::Framework,
        "Next.js",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "next.config.ts",
        TechnologyCategory::Framework,
        "Next.js",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "tailwind.config.ts",
        TechnologyCategory::Framework,
        "Tailwind CSS",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "tailwind.config.js",
        TechnologyCategory::Framework,
        "Tailwind CSS",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "prisma/schema.prisma",
        TechnologyCategory::Orm,
        "Prisma",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "dockerfile",
        TechnologyCategory::Infrastructure,
        "Docker",
        None,
        &mut technologies,
    );
    add_if(
        &names,
        "docker-compose.yml",
        TechnologyCategory::Infrastructure,
        "Docker Compose",
        None,
        &mut technologies,
    );

    add_package_technologies(root, files, &mut technologies);
    add_prisma_database_technologies(root, files, &mut technologies);

    if root.join("src-tauri").exists() {
        push_unique(
            &mut technologies,
            TechnologyCategory::Framework,
            "Tauri",
            None,
        );
    }

    technologies
}

fn add_package_technologies(
    root: &Path,
    files: &[RepositoryFile],
    technologies: &mut Vec<Technology>,
) {
    for file in files
        .iter()
        .filter(|file| file.path.to_lowercase().ends_with("package.json"))
    {
        let Ok(content) = fs::read_to_string(root.join(&file.path)) else {
            continue;
        };
        let Ok(package) = serde_json::from_str::<Value>(&content) else {
            continue;
        };
        for section_name in ["dependencies", "devDependencies"] {
            let Some(dependencies) = package.get(section_name).and_then(Value::as_object) else {
                continue;
            };
            for (dependency, version) in dependencies {
                let Some(version) = version.as_str() else {
                    continue;
                };
                if let Some((category, name)) = dependency_technology(dependency) {
                    push_unique(technologies, category, name, Some(version.to_string()));
                }
            }
        }
    }
}

fn dependency_technology(dependency: &str) -> Option<(TechnologyCategory, &'static str)> {
    match dependency {
        "react" | "react-dom" => Some((TechnologyCategory::Library, "React")),
        "next" => Some((TechnologyCategory::Framework, "Next.js")),
        "express" => Some((TechnologyCategory::Framework, "Express")),
        "@nestjs/core" => Some((TechnologyCategory::Framework, "NestJS")),
        "vite" => Some((TechnologyCategory::Framework, "Vite")),
        "tailwindcss" => Some((TechnologyCategory::Framework, "Tailwind CSS")),
        "@prisma/client" | "prisma" => Some((TechnologyCategory::Orm, "Prisma")),
        "typeorm" => Some((TechnologyCategory::Orm, "TypeORM")),
        "sequelize" => Some((TechnologyCategory::Orm, "Sequelize")),
        "drizzle-orm" => Some((TechnologyCategory::Orm, "Drizzle ORM")),
        "pg" | "postgres" => Some((TechnologyCategory::Database, "PostgreSQL")),
        "mysql" | "mysql2" => Some((TechnologyCategory::Database, "MySQL")),
        "sqlite3" | "better-sqlite3" => Some((TechnologyCategory::Database, "SQLite")),
        "mongodb" | "mongoose" => Some((TechnologyCategory::Database, "MongoDB")),
        "redis" | "ioredis" => Some((TechnologyCategory::Database, "Redis")),
        "@tanstack/react-query" => Some((TechnologyCategory::Library, "TanStack Query")),
        "zustand" => Some((TechnologyCategory::Library, "Zustand")),
        "axios" => Some((TechnologyCategory::Library, "Axios")),
        "zod" => Some((TechnologyCategory::Library, "Zod")),
        _ => None,
    }
}

fn add_prisma_database_technologies(
    root: &Path,
    files: &[RepositoryFile],
    technologies: &mut Vec<Technology>,
) {
    for file in files
        .iter()
        .filter(|file| file.path.to_lowercase().ends_with(".prisma"))
    {
        let Ok(content) = fs::read_to_string(root.join(&file.path)) else {
            continue;
        };
        for line in content.lines() {
            let Some(provider) = quoted_assignment_value(line.trim(), "provider") else {
                continue;
            };
            let database = match provider.as_str() {
                "postgresql" | "postgres" => Some("PostgreSQL"),
                "mysql" => Some("MySQL"),
                "sqlite" => Some("SQLite"),
                "mongodb" => Some("MongoDB"),
                "sqlserver" => Some("SQL Server"),
                "cockroachdb" => Some("CockroachDB"),
                _ => None,
            };
            if let Some(database) = database {
                push_unique(technologies, TechnologyCategory::Database, database, None);
            }
        }
    }
}

fn quoted_assignment_value(line: &str, key: &str) -> Option<String> {
    let (left, right) = line.split_once('=')?;
    if left.trim() != key {
        return None;
    }
    let value = right.trim();
    if value.len() < 2 {
        return None;
    }
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    value[1..]
        .find(quote)
        .map(|end| value[1..end + 1].to_string())
}

fn add_languages(files: &[RepositoryFile], technologies: &mut Vec<Technology>) {
    let extensions: BTreeSet<&str> = files
        .iter()
        .filter_map(|file| file.extension.as_deref())
        .collect();
    for (extension, language) in [
        ("ts", "TypeScript"),
        ("tsx", "TypeScript"),
        ("js", "JavaScript"),
        ("jsx", "JavaScript"),
        ("rs", "Rust"),
        ("py", "Python"),
        ("go", "Go"),
        ("java", "Java"),
        ("php", "PHP"),
        ("cs", "C#"),
    ] {
        if extensions.contains(extension) {
            push_unique(technologies, TechnologyCategory::Language, language, None);
        }
    }
}

fn add_if(
    names: &BTreeSet<String>,
    path: &str,
    category: TechnologyCategory,
    name: &str,
    version: Option<String>,
    technologies: &mut Vec<Technology>,
) {
    if names.contains(path) {
        push_unique(technologies, category, name, version);
    }
}

fn push_unique(
    technologies: &mut Vec<Technology>,
    category: TechnologyCategory,
    name: &str,
    version: Option<String>,
) {
    if let Some(existing) = technologies
        .iter()
        .find(|technology| technology.name == name && technology.category == category)
    {
        if existing.version.is_some() || version.is_none() {
            return;
        }
    }
    technologies.retain(|technology| technology.name != name || technology.category != category);
    technologies.push(Technology {
        category,
        name: name.to_string(),
        version,
    });
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join("/")
}

#[allow(dead_code)]
fn _path_buf_for_tests(path: &str) -> PathBuf {
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::{detect_technologies, ScanOptions, ScannerService};
    use devatlas_common::{RepositoryFile, RepositoryId, RepositoryPath, TechnologyCategory};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_vite_tauri_typescript_and_yarn() {
        let files = vec![
            file("package.json"),
            file("yarn.lock"),
            file("vite.config.ts"),
            file("src/main.tsx"),
            file("src-tauri/tauri.conf.json"),
        ];
        let technologies = detect_technologies(Path::new("."), &files);
        assert!(technologies
            .iter()
            .any(|technology| technology.name == "TypeScript"));
        assert!(technologies
            .iter()
            .any(|technology| technology.name == "Yarn"));
        assert!(technologies
            .iter()
            .any(|technology| technology.name == "Vite"));
        assert!(technologies
            .iter()
            .any(|technology| technology.name == "Tauri"));
        assert!(technologies
            .iter()
            .any(|technology| technology.category == TechnologyCategory::Language));
    }

    #[test]
    fn detects_package_libraries_orms_and_databases() {
        let root = unique_temp_dir("devatlas-scanner-package");
        fs::create_dir_all(root.join("prisma")).unwrap();
        fs::write(
            root.join("package.json"),
            r#"{"dependencies":{"react":"19.0.0","@prisma/client":"6.0.0","pg":"8.0.0","zustand":"5.0.0"}}"#,
        )
        .unwrap();
        fs::write(
            root.join("prisma/schema.prisma"),
            "datasource db {\n  provider = \"postgresql\"\n}\n",
        )
        .unwrap();

        let technologies =
            detect_technologies(&root, &[file("package.json"), file("prisma/schema.prisma")]);

        assert_technology(
            &technologies,
            TechnologyCategory::Library,
            "React",
            Some("19.0.0"),
        );
        assert_technology(
            &technologies,
            TechnologyCategory::Orm,
            "Prisma",
            Some("6.0.0"),
        );
        assert_technology(
            &technologies,
            TechnologyCategory::Database,
            "PostgreSQL",
            Some("8.0.0"),
        );
        assert_technology(
            &technologies,
            TechnologyCategory::Library,
            "Zustand",
            Some("5.0.0"),
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scans_source_files_without_build_output_by_default() {
        let root = unique_temp_dir("devatlas-scanner-source");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::create_dir_all(root.join("dist")).unwrap();
        fs::write(root.join("src/main.ts"), "export const value = 1;").unwrap();
        fs::write(root.join("src/feature.ts"), "export const feature = 1;").unwrap();
        fs::write(
            root.join("node_modules/pkg/index.js"),
            "module.exports = {};",
        )
        .unwrap();
        fs::write(root.join("dist/bundle.js"), "bundle").unwrap();

        let repository_path = RepositoryPath::new(root.clone()).unwrap();
        let scan =
            ScannerService::scan_repository(RepositoryId("repo-1".to_string()), &repository_path)
                .unwrap();

        assert_eq!(scan.files_count, 2);
        assert!(scan.files.iter().any(|file| file.path == "src/main.ts"));
        assert!(scan.files.iter().any(|file| file.path == "src/feature.ts"));
        assert!(!scan
            .files
            .iter()
            .any(|file| file.path.contains("node_modules")));
        assert!(!scan.files.iter().any(|file| file.path.contains("dist")));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_options_can_limit_source_file_count() {
        let root = unique_temp_dir("devatlas-scanner-limit");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/a.ts"), "a").unwrap();
        fs::write(root.join("src/b.ts"), "b").unwrap();
        fs::write(root.join("src/c.ts"), "c").unwrap();

        let repository_path = RepositoryPath::new(root.clone()).unwrap();
        let scan = ScannerService::scan_repository_with_options(
            RepositoryId("repo-1".to_string()),
            &repository_path,
            &ScanOptions {
                max_files: Some(2),
                include_paths: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(scan.files_count, 2);

        fs::remove_dir_all(root).unwrap();
    }

    fn assert_technology(
        technologies: &[devatlas_common::Technology],
        category: TechnologyCategory,
        name: &str,
        version: Option<&str>,
    ) {
        assert!(technologies.iter().any(|technology| {
            technology.category == category
                && technology.name == name
                && technology.version.as_deref() == version
        }));
    }

    fn file(path: &str) -> RepositoryFile {
        RepositoryFile {
            path: path.to_string(),
            extension: path.rsplit('.').next().map(ToString::to_string),
            size_bytes: 1,
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
