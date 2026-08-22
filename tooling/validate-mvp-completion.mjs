import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = dirname(dirname(fileURLToPath(import.meta.url)));

const completionDoc = readText("docs/mvp-completion.md");
const contractsSource = readText("apps/desktop/src/types/contracts.ts");
const navigationSource = readText("apps/desktop/src/handlers/navigation.tsx");
const cargoToml = readText("Cargo.toml");
const docsEngine = readText("crates/docs-engine/src/lib.rs");
const umlEngine = readText("crates/uml-engine/src/lib.rs");
const exportEngine = readText("crates/export-engine/src/lib.rs");
const roadmapStatus = JSON.parse(readText("docs/roadmap-status.json"));
const packageJson = JSON.parse(readText("package.json"));
const readme = readText("README.md");

const requiredPages = [
  "dashboard",
  "explorer",
  "scanner",
  "documentation",
  "diagrams",
  "exports",
  "graphs",
  "settings",
];
const requiredCommands = [
  "open_repository",
  "scan_repository",
  "list_repository_files",
  "detect_technologies",
  "build_graph",
  "generate_docs",
  "generate_diagrams",
  "export_package",
];
const requiredCrates = [
  "crates/common",
  "crates/storage-engine",
  "crates/app-core",
  "crates/scanner-engine",
  "crates/parser-engine",
  "crates/graph-engine",
  "crates/docs-engine",
  "crates/uml-engine",
  "crates/export-engine",
];
const requiredDocumentArtifacts = [
  "docs/README.md",
  "docs/architecture.md",
  "docs/developer-guide.md",
  "docs/api-summary.md",
  "docs/database-summary.md",
  "docs/onboarding.md",
  "docs/ai-context.md",
];
const requiredDiagramArtifacts = [
  "diagrams/class.puml",
  "diagrams/component.mmd",
  "diagrams/dependency.puml",
  "diagrams/erd.mmd",
  "diagrams/folder-structure.svg",
  "diagrams/package.mmd",
  "diagrams/architecture-overview.mmd",
];
const requiredExportArtifacts = [
  "repository-summary.json",
  "export-manifest.json",
  "project-knowledge.zip",
];
const futureRuntimeFlags = [
  "cloudRuntimeEnabled",
  "mcpRuntimeEnabled",
  "pluginRuntimeEnabled",
  "securityRuntimeEnabled",
  "performanceRuntimeEnabled",
  "gitRuntimeEnabled",
];

assertContainsAll("MVP completion doc is missing required user-flow commands.", completionDoc, requiredCommands);
assertContainsAll("MVP completion doc is missing required desktop pages.", completionDoc, requiredPages.map(titleCase));
assertContainsAll("MVP completion doc is missing documentation artifacts.", completionDoc, requiredDocumentArtifacts);
assertContainsAll("MVP completion doc is missing diagram artifacts.", completionDoc, requiredDiagramArtifacts);
assertContainsAll("MVP completion doc is missing export artifacts.", completionDoc, requiredExportArtifacts);

const typedCommands = extractTauriCommands(contractsSource);
const missingCommands = requiredCommands.filter((command) => !typedCommands.includes(command));
if (missingCommands.length > 0) {
  reportFailure("TAURI_COMMANDS is missing required MVP commands.", missingCommands);
}

const missingPages = requiredPages.filter((page) => !navigationSource.includes(`page: "${page}"`));
if (missingPages.length > 0) {
  reportFailure("Desktop navigation is missing required MVP pages.", missingPages);
}

const missingCrates = requiredCrates.filter((cratePath) => !cargoToml.includes(`"${cratePath}"`));
if (missingCrates.length > 0) {
  reportFailure("Cargo workspace is missing required MVP crates.", missingCrates);
}

const missingDocumentOutputs = requiredDocumentArtifacts.filter((artifact) => !docsEngine.includes(artifact));
if (missingDocumentOutputs.length > 0) {
  reportFailure("Documentation engine is missing required MVP output paths.", missingDocumentOutputs);
}

const missingDiagramOutputs = requiredDiagramArtifacts.filter((artifact) => !umlEngine.includes(artifact));
if (missingDiagramOutputs.length > 0) {
  reportFailure("UML engine is missing required MVP output paths.", missingDiagramOutputs);
}

const missingExportOutputs = requiredExportArtifacts.filter((artifact) => !exportEngine.includes(artifact));
if (missingExportOutputs.length > 0) {
  reportFailure("Export engine is missing required MVP package artifacts.", missingExportOutputs);
}

if (!roadmapStatus.runtimePolicy?.desktopFirst || !roadmapStatus.runtimePolicy?.localOnly) {
  reportFailure("MVP runtime policy must remain desktop-first and local-only.", [
    "runtimePolicy.desktopFirst and runtimePolicy.localOnly must both be true.",
  ]);
}

const enabledFutureFlags = futureRuntimeFlags.filter((flag) => roadmapStatus.runtimePolicy?.[flag] !== false);
if (enabledFutureFlags.length > 0) {
  reportFailure("Future runtime flags must remain disabled for MVP completion.", enabledFutureFlags);
}

if (!String(packageJson.scripts?.validate).includes("yarn mvp:check")) {
  reportFailure("Root validate script must include MVP completion validation.", [
    "Expected `yarn mvp:check` inside package.json scripts.validate.",
  ]);
}

if (!readme.includes("yarn mvp:check") || !readme.includes("docs/mvp-completion.md")) {
  reportFailure("README must document the MVP completion gate.", [
    "Missing yarn mvp:check or docs/mvp-completion.md.",
  ]);
}

console.log("mvp completion ok");

function readText(relativePath) {
  return readFileSync(join(workspaceRoot, relativePath), "utf8");
}

function extractTauriCommands(source) {
  const match = source.match(/TAURI_COMMANDS\s*=\s*\[([\s\S]*?)\]\s+as const/);
  if (!match) {
    reportFailure("Could not locate TAURI_COMMANDS in frontend contract source.", [
      "apps/desktop/src/types/contracts.ts",
    ]);
  }
  return [...match[1].matchAll(/"([^"]+)"/g)].map((commandMatch) => commandMatch[1]);
}

function assertContainsAll(message, source, requiredValues) {
  const missingValues = requiredValues.filter((value) => !source.includes(value));
  if (missingValues.length > 0) {
    reportFailure(message, missingValues);
  }
}

function titleCase(value) {
  return value
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function reportFailure(message, details) {
  console.error(message);
  for (const detail of details) {
    console.error(`- ${detail}`);
  }
  process.exit(1);
}
