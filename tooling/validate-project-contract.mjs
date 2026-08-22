import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = dirname(dirname(fileURLToPath(import.meta.url)));

const sourceDocuments = [
  "plan/00-vision.md",
  "plan/01-product-prd.md",
  "plan/02-system-architecture.md",
  "plan/03-desktop-app-spec.md",
  "plan/04-repository-scanner-spec.md",
  "plan/05-ai-chat-rag-spec.md",
  "plan/06-knowledge-graph-spec.md",
  "plan/07-uml-engine-spec.md",
  "plan/08-git-intelligence-spec.md",
  "plan/09-documentation-engine-spec.md",
  "plan/10-security-analysis-spec.md",
  "plan/11-performance-analysis-spec.md",
  "plan/12-export-engine-spec.md",
  "plan/13-plugin-system-spec.md",
  "plan/14-roadmap.md",
  "plan/15-master-prompt.md",
  "plan/27-mvp-cutdown.md",
  "plan/31-implementation-blueprint.md",
  "plan/index.md",
];

const requiredRuntimeFiles = [
  "apps/desktop/src/main.tsx",
  "apps/desktop/src/services/commands.ts",
  "apps/desktop/src-tauri/src/lib.rs",
  "crates/app-core/src/lib.rs",
  "crates/common/src/lib.rs",
  "crates/storage-engine/src/lib.rs",
];

const mvpCrates = [
  "common",
  "storage-engine",
  "app-core",
  "scanner-engine",
  "parser-engine",
  "graph-engine",
  "docs-engine",
  "uml-engine",
  "export-engine",
];

const futureRuntimeCrates = [
  "git-engine",
  "security-engine",
  "performance-engine",
  "plugin-engine",
  "mcp-server",
  "cloud-platform",
];

const missingSourceDocuments = sourceDocuments.filter((path) => !existsSync(join(workspaceRoot, path)));
const missingRuntimeFiles = requiredRuntimeFiles.filter((path) => !existsSync(join(workspaceRoot, path)));

if (missingSourceDocuments.length > 0 || missingRuntimeFiles.length > 0) {
  reportFailure("Project contract source files are missing.", [
    ...missingSourceDocuments.map((path) => `Missing source document: ${path}`),
    ...missingRuntimeFiles.map((path) => `Missing runtime file: ${path}`),
  ]);
}

const packageJson = JSON.parse(readText("package.json"));
if (packageJson.packageManager !== "yarn@1.22.22") {
  reportFailure("Project contract requires Yarn as the active Node.js package manager.", [
    `Expected yarn@1.22.22, received ${String(packageJson.packageManager)}`,
  ]);
}

const cargoToml = readText("Cargo.toml");
const missingMvpCrates = mvpCrates.filter((crateName) => !cargoToml.includes(`"crates/${crateName}"`));
if (missingMvpCrates.length > 0) {
  reportFailure("Cargo workspace is missing MVP runtime crates.", missingMvpCrates);
}

const enabledFutureCrates = futureRuntimeCrates.filter((crateName) =>
  cargoToml.includes(`"crates/${crateName}"`),
);
if (enabledFutureCrates.length > 0) {
  reportFailure("Future runtime crates must not be enabled without an explicit implementation task.", enabledFutureCrates);
}

const commandsSource = readText("apps/desktop/src/services/commands.ts");
if (!commandsSource.includes("invokeCommand<TResponse>")) {
  reportFailure("React must call Rust through the typed Tauri command helper.", [
    "apps/desktop/src/services/commands.ts is missing invokeCommand<TResponse>.",
  ]);
}

const appCoreSource = readText("crates/app-core/src/lib.rs");
for (const crateName of mvpCrates) {
  const rustName = `devatlas_${crateName.replaceAll("-", "_")}`;
  if (!["common", "app-core"].includes(crateName) && !appCoreSource.includes(rustName)) {
    reportFailure("App core must orchestrate MVP Rust engine crates.", [
      `Missing app-core reference to ${rustName}.`,
    ]);
  }
}

const desktopSource = readText("apps/desktop/src-tauri/src/lib.rs");
if (!desktopSource.includes("tauri::generate_handler!")) {
  reportFailure("Desktop backend must expose behavior through Tauri commands.", [
    "apps/desktop/src-tauri/src/lib.rs is missing tauri::generate_handler!.",
  ]);
}

const storageSource = readText("crates/storage-engine/src/lib.rs");
if (!storageSource.includes("rusqlite")) {
  reportFailure("Storage contract requires SQLite through rusqlite.", [
    "crates/storage-engine/src/lib.rs does not reference rusqlite.",
  ]);
}

console.log("project contract ok");

function readText(relativePath) {
  return readFileSync(join(workspaceRoot, relativePath), "utf8");
}

function reportFailure(message, details) {
  console.error(message);
  for (const detail of details) {
    console.error(`- ${detail}`);
  }
  process.exit(1);
}
