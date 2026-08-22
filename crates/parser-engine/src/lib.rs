use devatlas_common::{stable_id, DevAtlasResult, RepositoryFile, RepositoryPath};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModule {
    pub name: String,
    pub path: String,
    pub module_type: ModuleType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleType {
    Source,
    Test,
    Configuration,
    Documentation,
    Asset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSymbol {
    pub id: String,
    pub name: String,
    pub path: String,
    pub symbol_type: SymbolType,
    pub line: usize,
    pub owner_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Trait,
    Module,
}

impl SymbolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "Function",
            Self::Method => "Method",
            Self::Class => "Class",
            Self::Struct => "Struct",
            Self::Interface => "Interface",
            Self::Trait => "Trait",
            Self::Module => "Module",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRelationship {
    pub source_path: String,
    pub target: String,
    pub relationship_type: RelationshipType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipType {
    Imports,
    Calls,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Imports => "Imports",
            Self::Calls => "Calls",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRepository {
    pub modules: Vec<ParsedModule>,
    pub symbols: Vec<ParsedSymbol>,
    pub relationships: Vec<ParsedRelationship>,
}

pub struct ParserService;

impl ParserService {
    pub fn extract_modules(files: &[RepositoryFile]) -> Vec<ParsedModule> {
        files.iter().map(classify_file).collect()
    }

    pub fn parse_repository(
        repository_path: &RepositoryPath,
        files: &[RepositoryFile],
    ) -> DevAtlasResult<ParsedRepository> {
        let modules = Self::extract_modules(files);
        let mut symbols = Vec::new();
        let mut relationships = Vec::new();
        let source_paths = files
            .iter()
            .filter(|file| is_supported_source(file))
            .map(|file| file.path.clone())
            .collect::<BTreeSet<String>>();
        let mut source_contents = BTreeMap::new();

        for file in files.iter().filter(|file| is_supported_source(file)) {
            let absolute_path = repository_path.as_path().join(&file.path);
            if file.size_bytes > 1_048_576 {
                continue;
            }
            let Ok(content) = fs::read_to_string(&absolute_path) else {
                continue;
            };
            parse_source_symbols_and_imports(
                &file.path,
                &content,
                &source_paths,
                &mut symbols,
                &mut relationships,
            );
            source_contents.insert(file.path.clone(), content);
        }
        add_call_relationships(&source_contents, &symbols, &mut relationships);

        Ok(ParsedRepository {
            modules,
            symbols,
            relationships,
        })
    }
}

fn classify_file(file: &RepositoryFile) -> ParsedModule {
    let lower_path = file.path.to_lowercase();
    let module_type = if lower_path.contains("/test")
        || lower_path.ends_with(".test.ts")
        || lower_path.ends_with(".spec.ts")
    {
        ModuleType::Test
    } else if lower_path.ends_with(".md") {
        ModuleType::Documentation
    } else if lower_path.ends_with(".json")
        || lower_path.ends_with(".toml")
        || lower_path.ends_with(".yaml")
        || lower_path.ends_with(".yml")
    {
        ModuleType::Configuration
    } else if lower_path.ends_with(".png")
        || lower_path.ends_with(".svg")
        || lower_path.ends_with(".jpg")
    {
        ModuleType::Asset
    } else {
        ModuleType::Source
    };
    ParsedModule {
        name: file
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&file.path)
            .to_string(),
        path: file.path.clone(),
        module_type,
    }
}

fn is_supported_source(file: &RepositoryFile) -> bool {
    matches!(
        file.extension.as_deref(),
        Some("ts" | "tsx" | "js" | "jsx" | "rs" | "py" | "go" | "java" | "php" | "cs")
    )
}

fn parse_source_symbols_and_imports(
    path: &str,
    content: &str,
    source_paths: &BTreeSet<String>,
    symbols: &mut Vec<ParsedSymbol>,
    relationships: &mut Vec<ParsedRelationship>,
) {
    let mut brace_depth = 0_isize;
    let mut active_container: Option<(String, isize)> = None;
    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if let Some((symbol_type, name)) = extract_symbol(trimmed) {
            let id = stable_id("symbol", &format!("{path}-{name}-{}", line_index + 1));
            let owner_id = if symbol_type == SymbolType::Method {
                active_container
                    .as_ref()
                    .map(|(container_id, _depth)| container_id.clone())
            } else {
                None
            };
            symbols.push(ParsedSymbol {
                id: id.clone(),
                name,
                path: path.to_string(),
                symbol_type: symbol_type.clone(),
                line: line_index + 1,
                owner_id,
            });
            if is_container_symbol(&symbol_type) && trimmed.contains('{') {
                active_container = Some((id, brace_depth));
            }
        }
        if let Some(target) = extract_import_target(trimmed) {
            relationships.push(ParsedRelationship {
                source_path: path.to_string(),
                target: resolve_import_target(path, &target, source_paths).unwrap_or(target),
                relationship_type: RelationshipType::Imports,
            });
        }
        brace_depth += brace_delta(trimmed);
        if active_container
            .as_ref()
            .is_some_and(|(_container_id, container_depth)| brace_depth <= *container_depth)
        {
            active_container = None;
        }
    }
}

fn add_call_relationships(
    source_contents: &BTreeMap<String, String>,
    symbols: &[ParsedSymbol],
    relationships: &mut Vec<ParsedRelationship>,
) {
    let symbol_names = symbols
        .iter()
        .filter(|symbol| {
            matches!(
                symbol.symbol_type,
                SymbolType::Function | SymbolType::Method
            )
        })
        .map(|symbol| (symbol.name.as_str(), symbol.id.as_str()))
        .collect::<BTreeMap<&str, &str>>();
    if symbol_names.is_empty() {
        return;
    }

    let mut seen = relationships
        .iter()
        .map(|relationship| {
            (
                relationship.source_path.clone(),
                relationship.target.clone(),
                relationship.relationship_type.as_str().to_string(),
            )
        })
        .collect::<BTreeSet<(String, String, String)>>();

    for (path, content) in source_contents {
        for line in content.lines().map(str::trim) {
            let defined_symbol_name = extract_symbol(line).map(|(_symbol_type, name)| name);
            for (name, symbol_id) in &symbol_names {
                if defined_symbol_name.as_deref() == Some(name) {
                    continue;
                }
                if line_contains_call(line, name) {
                    let key = (path.clone(), (*symbol_id).to_string(), "Calls".to_string());
                    if seen.insert(key) {
                        relationships.push(ParsedRelationship {
                            source_path: path.clone(),
                            target: (*symbol_id).to_string(),
                            relationship_type: RelationshipType::Calls,
                        });
                    }
                }
            }
        }
    }
}

fn extract_symbol(line: &str) -> Option<(SymbolType, String)> {
    if let Some(rest) = line.strip_prefix("type ") {
        let name = take_identifier(rest);
        if !name.is_empty() {
            let symbol_type = if rest.contains(" interface") {
                SymbolType::Interface
            } else {
                SymbolType::Struct
            };
            return Some((symbol_type, name));
        }
    }

    for (prefix, symbol_type) in [
        ("export async function ", SymbolType::Function),
        ("export function ", SymbolType::Function),
        ("async function ", SymbolType::Function),
        ("function ", SymbolType::Function),
        ("export class ", SymbolType::Class),
        ("class ", SymbolType::Class),
        ("export interface ", SymbolType::Interface),
        ("interface ", SymbolType::Interface),
        ("pub struct ", SymbolType::Struct),
        ("struct ", SymbolType::Struct),
        ("pub trait ", SymbolType::Trait),
        ("trait ", SymbolType::Trait),
        ("pub fn ", SymbolType::Function),
        ("fn ", SymbolType::Function),
        ("def ", SymbolType::Function),
        ("func ", SymbolType::Function),
        ("public function ", SymbolType::Function),
        ("protected function ", SymbolType::Function),
        ("private function ", SymbolType::Function),
        ("final class ", SymbolType::Class),
        ("abstract class ", SymbolType::Class),
        ("static class ", SymbolType::Class),
        ("public class ", SymbolType::Class),
        ("private class ", SymbolType::Class),
        ("protected class ", SymbolType::Class),
        ("public interface ", SymbolType::Interface),
        ("private interface ", SymbolType::Interface),
        ("protected interface ", SymbolType::Interface),
        ("public struct ", SymbolType::Struct),
        ("private struct ", SymbolType::Struct),
        ("protected struct ", SymbolType::Struct),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = take_identifier(rest);
            if !name.is_empty() {
                return Some((symbol_type, name));
            }
        }
    }
    if let Some(name) = extract_arrow_function_name(line) {
        return Some((SymbolType::Function, name));
    }
    if let Some(name) = extract_method_name(line) {
        return Some((SymbolType::Method, name));
    }
    None
}

fn is_container_symbol(symbol_type: &SymbolType) -> bool {
    matches!(
        symbol_type,
        SymbolType::Class | SymbolType::Struct | SymbolType::Interface | SymbolType::Trait
    )
}

fn brace_delta(line: &str) -> isize {
    let opens = line.chars().filter(|character| *character == '{').count() as isize;
    let closes = line.chars().filter(|character| *character == '}').count() as isize;
    opens - closes
}

fn extract_arrow_function_name(line: &str) -> Option<String> {
    for prefix in ["export const ", "const ", "let ", "var "] {
        let Some(rest) = line.strip_prefix(prefix) else {
            continue;
        };
        let (left, right) = rest.split_once('=')?;
        if !right.contains("=>") && !right.trim_start().starts_with("function") {
            continue;
        }
        let name = take_identifier(left.trim());
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn extract_method_name(line: &str) -> Option<String> {
    if line.ends_with(';') || line.starts_with("//") {
        return None;
    }
    let without_access = strip_any_prefix(
        line,
        &[
            "public async ",
            "private async ",
            "protected async ",
            "public static ",
            "private static ",
            "protected static ",
            "public ",
            "private ",
            "protected ",
            "async ",
            "static ",
        ],
    );
    let method_name = without_access
        .split_once('(')
        .map(|(name, _rest)| take_identifier(name.trim()))?;
    if method_name.is_empty() || is_control_keyword(&method_name) {
        return None;
    }
    let rest = without_access[method_name.len()..].trim_start();
    if rest.starts_with('(') {
        return Some(method_name);
    }
    let arrow_assignment = without_access
        .split_once('=')
        .map(|(name, right)| (take_identifier(name.trim()), right.trim()))?;
    if !arrow_assignment.0.is_empty() && arrow_assignment.1.contains("=>") {
        return Some(arrow_assignment.0);
    }
    None
}

fn strip_any_prefix<'a>(value: &'a str, prefixes: &[&str]) -> &'a str {
    prefixes
        .iter()
        .find_map(|prefix| value.strip_prefix(prefix))
        .unwrap_or(value)
}

fn is_control_keyword(value: &str) -> bool {
    matches!(
        value,
        "if" | "for" | "while" | "switch" | "catch" | "return" | "match"
    )
}

fn extract_import_target(line: &str) -> Option<String> {
    if line.starts_with("import ") || line.starts_with("export ") {
        return quoted_value(line).or_else(|| {
            line.strip_prefix("import ")
                .map(|value| value.trim_end_matches(';').trim().to_string())
                .filter(|value| !value.is_empty() && value != "(")
        });
    }
    if let Some(rest) = line.strip_prefix("use ") {
        return Some(
            rest.trim_end_matches(';')
                .split("::")
                .next()
                .unwrap_or(rest)
                .to_string(),
        );
    }
    if let Some(rest) = line.strip_prefix("from ") {
        return rest
            .split_whitespace()
            .next()
            .map(|value| value.trim_matches('"').trim_matches('\'').to_string());
    }
    if let Some(rest) = line.strip_prefix("using ") {
        return Some(rest.trim_end_matches(';').trim().to_string());
    }
    None
}

fn resolve_import_target(
    source_path: &str,
    target: &str,
    source_paths: &BTreeSet<String>,
) -> Option<String> {
    if !target.starts_with('.') {
        return None;
    }
    let source_directory = source_path
        .rsplit_once('/')
        .map(|(directory, _)| directory)?;
    let base = normalize_relative_path(&format!("{source_directory}/{target}"))?;
    let candidates = [
        base.clone(),
        format!("{base}.ts"),
        format!("{base}.tsx"),
        format!("{base}.js"),
        format!("{base}.jsx"),
        format!("{base}.rs"),
        format!("{base}/index.ts"),
        format!("{base}/index.tsx"),
        format!("{base}/index.js"),
        format!("{base}/index.jsx"),
    ];
    candidates
        .into_iter()
        .find(|candidate| source_paths.contains(candidate))
}

fn normalize_relative_path(path: &str) -> Option<String> {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                parts.pop()?;
            }
            value => parts.push(value),
        }
    }
    Some(parts.join("/"))
}

fn line_contains_call(line: &str, symbol_name: &str) -> bool {
    let mut search_start = 0;
    while let Some(relative_index) = line[search_start..].find(symbol_name) {
        let index = search_start + relative_index;
        let before = line[..index].chars().last();
        let after_name = &line[index + symbol_name.len()..];
        let has_identifier_boundary =
            !before.is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        if has_identifier_boundary && after_name.trim_start().starts_with('(') {
            return true;
        }
        search_start = index + symbol_name.len();
    }
    false
}

fn quoted_value(line: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(start) = line.find(quote) {
            let remaining = &line[start + 1..];
            if let Some(end) = remaining.find(quote) {
                return Some(remaining[..end].to_string());
            }
        }
    }
    None
}

fn take_identifier(value: &str) -> String {
    value
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        extract_import_target, extract_symbol, ModuleType, ParserService, RelationshipType,
        SymbolType,
    };
    use devatlas_common::{RepositoryFile, RepositoryPath};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn classifies_test_file() {
        let modules = ParserService::extract_modules(&[RepositoryFile {
            path: "src/app.test.ts".to_string(),
            extension: Some("ts".to_string()),
            size_bytes: 1,
        }]);
        assert_eq!(modules[0].module_type, ModuleType::Test);
    }

    #[test]
    fn extracts_typescript_function_and_import() {
        let symbol = extract_symbol("export function scanRepository() {").unwrap();
        assert_eq!(symbol, (SymbolType::Function, "scanRepository".to_string()));
        assert_eq!(
            extract_symbol("export const loadRepository = async () => {};"),
            Some((SymbolType::Function, "loadRepository".to_string()))
        );
        assert_eq!(
            extract_symbol("public async refreshRepository() {"),
            Some((SymbolType::Method, "refreshRepository".to_string()))
        );
        assert_eq!(
            extract_import_target("import { invoke } from \"@tauri-apps/api/core\";"),
            Some("@tauri-apps/api/core".to_string())
        );
    }

    #[test]
    fn extracts_rust_struct_and_use() {
        let symbol = extract_symbol("pub struct ScannerService;").unwrap();
        assert_eq!(symbol, (SymbolType::Struct, "ScannerService".to_string()));
        assert_eq!(
            extract_import_target("use devatlas_common::ScanResult;"),
            Some("devatlas_common".to_string())
        );
    }

    #[test]
    fn extracts_go_symbols_and_imports() {
        assert_eq!(
            extract_symbol("type Repository interface {"),
            Some((SymbolType::Interface, "Repository".to_string()))
        );
        assert_eq!(
            extract_symbol("func ScanRepository() error {"),
            Some((SymbolType::Function, "ScanRepository".to_string()))
        );
        assert_eq!(
            extract_import_target("import \"github.com/example/project\""),
            Some("github.com/example/project".to_string())
        );
    }

    #[test]
    fn extracts_java_php_and_csharp_symbols_and_imports() {
        assert_eq!(
            extract_symbol("public class UserController {"),
            Some((SymbolType::Class, "UserController".to_string()))
        );
        assert_eq!(
            extract_import_target("import java.util.List;"),
            Some("java.util.List".to_string())
        );
        assert_eq!(
            extract_symbol("public function handleRequest() {"),
            Some((SymbolType::Function, "handleRequest".to_string()))
        );
        assert_eq!(
            extract_import_target("using System.Collections.Generic;"),
            Some("System.Collections.Generic".to_string())
        );
    }

    #[test]
    fn skips_invalid_source_files_and_continues_parsing() {
        let root = unique_temp_dir("devatlas-parser-invalid-source");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/valid.ts"),
            "export function validFunction() {}\n",
        )
        .unwrap();
        fs::write(root.join("src/invalid.ts"), [0xff, 0xfe, 0xfd]).unwrap();
        let repository_path = RepositoryPath::new(&root).unwrap();
        let parsed = ParserService::parse_repository(
            &repository_path,
            &[file("src/valid.ts", 35), file("src/invalid.ts", 3)],
        )
        .unwrap();

        assert!(parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "validFunction"));
        assert!(parsed
            .modules
            .iter()
            .any(|module| module.path == "src/invalid.ts"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_relative_imports_and_function_calls() {
        let root = unique_temp_dir("devatlas-parser-relationships");
        fs::create_dir_all(root.join("src/api")).unwrap();
        fs::write(
            root.join("src/api/app.ts"),
            "import { helper } from '../helper';\nexport function routeHandler() { return helper(); }\n",
        )
        .unwrap();
        fs::write(
            root.join("src/helper.ts"),
            "export function helper() { return true; }\n",
        )
        .unwrap();
        let repository_path = RepositoryPath::new(&root).unwrap();
        let parsed = ParserService::parse_repository(
            &repository_path,
            &[file("src/api/app.ts", 80), file("src/helper.ts", 42)],
        )
        .unwrap();

        assert!(parsed.relationships.iter().any(|relationship| {
            relationship.source_path == "src/api/app.ts"
                && relationship.target == "src/helper.ts"
                && relationship.relationship_type == RelationshipType::Imports
        }));
        let helper = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.name == "helper")
            .expect("helper symbol should be parsed");
        assert!(parsed.relationships.iter().any(|relationship| {
            relationship.source_path == "src/api/app.ts"
                && relationship.target == helper.id.as_str()
                && relationship.relationship_type == RelationshipType::Calls
        }));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn assigns_method_owner_inside_class_body() {
        let root = unique_temp_dir("devatlas-parser-method-owner");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/service.ts"),
            "export class AppService {\n  public async refreshRepository() { return true; }\n}\n",
        )
        .unwrap();
        let repository_path = RepositoryPath::new(&root).unwrap();
        let parsed =
            ParserService::parse_repository(&repository_path, &[file("src/service.ts", 80)])
                .unwrap();
        let class_symbol = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.name == "AppService")
            .expect("class symbol should be parsed");
        let method_symbol = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.name == "refreshRepository")
            .expect("method symbol should be parsed");

        assert_eq!(
            method_symbol.owner_id.as_deref(),
            Some(class_symbol.id.as_str())
        );

        fs::remove_dir_all(root).unwrap();
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
