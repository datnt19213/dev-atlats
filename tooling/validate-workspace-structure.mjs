import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = dirname(dirname(fileURLToPath(import.meta.url)));

const requiredDirectories = [
  "apps",
  ".github",
  ".github/workflows",
  "apps/desktop",
  "apps/desktop/src",
  "apps/desktop/src/app",
  "apps/desktop/src/components",
  "apps/desktop/src/components/ui",
  "apps/desktop/src/features",
  "apps/desktop/src/lib",
  "apps/desktop/src/services",
  "apps/desktop/src/stores",
  "apps/desktop/src/styles",
  "apps/desktop/src/test",
  "apps/desktop/src/types",
  "apps/desktop/src-tauri",
  "apps/desktop/src-tauri/capabilities",
  "crates",
  "crates/common",
  "crates/ai-engine",
  "crates/storage-engine",
  "crates/app-core",
  "crates/scanner-engine",
  "crates/parser-engine",
  "crates/graph-engine",
  "crates/docs-engine",
  "crates/uml-engine",
  "crates/export-engine",
  "docs",
  "plan",
  "resources",
  "scripts",
  "tests",
  "tooling",
];

const requiredFiles = [
  "package.json",
  "yarn.lock",
  "Cargo.toml",
  "Dockerfile",
  "docker-compose.yml",
  ".github/workflows/validate.yml",
  "README.md",
  "docs/mvp-completion.md",
  "docs/release-playbook.md",
  "postman_collection.txt",
  "apps/desktop/package.json",
  "apps/desktop/vite.config.ts",
  "apps/desktop/tsconfig.json",
  "apps/desktop/Dockerfile",
  "apps/desktop/docker-compose.yml",
  "scripts/generate-release-checksums.mjs",
  "tooling/validate-mvp-completion.mjs",
  "apps/desktop/src/main.tsx",
  "apps/desktop/src-tauri/Cargo.toml",
  "apps/desktop/src-tauri/capabilities/default.json",
  "apps/desktop/src-tauri/src/lib.rs",
];

const expectedCargoMembers = [
  "crates/common",
  "crates/ai-engine",
  "crates/storage-engine",
  "crates/app-core",
  "crates/scanner-engine",
  "crates/parser-engine",
  "crates/graph-engine",
  "crates/docs-engine",
  "crates/uml-engine",
  "crates/export-engine",
  "apps/desktop/src-tauri",
];

const missingDirectories = requiredDirectories.filter((relativePath) => !isDirectory(relativePath));
const missingFiles = requiredFiles.filter((relativePath) => !isFile(relativePath));

if (missingDirectories.length > 0 || missingFiles.length > 0) {
  reportFailure("Workspace structure is missing required MVP paths.", [
    ...missingDirectories.map((path) => `Missing directory: ${path}`),
    ...missingFiles.map((path) => `Missing file: ${path}`),
  ]);
}

const packageJson = JSON.parse(readText("package.json"));
if (packageJson.packageManager !== "yarn@1.22.22") {
  reportFailure("Workspace package manager must remain Yarn.", [
    `Expected packageManager yarn@1.22.22, received ${String(packageJson.packageManager)}`,
  ]);
}

if (!Array.isArray(packageJson.workspaces) || !packageJson.workspaces.includes("apps/*")) {
  reportFailure("Workspace package.json must expose apps/* as a Yarn workspace.", [
    "Missing apps/* in package.json workspaces.",
  ]);
}

const cargoToml = readText("Cargo.toml");
const missingCargoMembers = expectedCargoMembers.filter(
  (member) => !cargoToml.includes(`"${member}"`),
);

if (missingCargoMembers.length > 0) {
  reportFailure("Cargo workspace is missing required MVP members.", missingCargoMembers);
}

console.log("workspace structure ok");

function readText(relativePath) {
  return readFileSync(join(workspaceRoot, relativePath), "utf8");
}

function isDirectory(relativePath) {
  const absolutePath = join(workspaceRoot, relativePath);
  return existsSync(absolutePath) && statSync(absolutePath).isDirectory();
}

function isFile(relativePath) {
  const absolutePath = join(workspaceRoot, relativePath);
  return existsSync(absolutePath) && statSync(absolutePath).isFile();
}

function reportFailure(message, details) {
  console.error(message);
  for (const detail of details) {
    console.error(`- ${detail}`);
  }
  process.exit(1);
}
