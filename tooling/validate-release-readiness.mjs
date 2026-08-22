import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = dirname(dirname(fileURLToPath(import.meta.url)));

const rootPackage = readJson("package.json");
const desktopPackage = readJson("apps/desktop/package.json");
const tauriConfig = readJson("apps/desktop/src-tauri/tauri.conf.json");
const postmanCollection = readJson("postman_collection.txt");
const contractsSource = readText("apps/desktop/src/types/contracts.ts");
const rootDockerfile = readText("Dockerfile");
const desktopDockerfile = readText("apps/desktop/Dockerfile");
const rootCompose = readText("docker-compose.yml");
const desktopCompose = readText("apps/desktop/docker-compose.yml");
const githubWorkflow = readText(".github/workflows/validate.yml");
const releasePlaybook = readText("docs/release-playbook.md");
const checksumScript = readText("scripts/generate-release-checksums.mjs");
const readme = readText("README.md");

const semverPattern = /^\d+\.\d+\.\d+$/;
const requiredRootScripts = [
  "structure:check",
  "contract:check",
  "mvp:check",
  "roadmap:check",
  "release:check",
  "release:checksums",
  "format:check",
  "lint",
  "typecheck",
  "test",
  "build",
  "validate",
];
const requiredDesktopScripts = ["lint", "typecheck", "test", "build", "tauri"];

if (!semverPattern.test(String(rootPackage.version))) {
  reportFailure("Root package version must use semantic versioning.", [
    `Received ${String(rootPackage.version)}`,
  ]);
}

if (rootPackage.version !== desktopPackage.version || rootPackage.version !== tauriConfig.version) {
  reportFailure("Release versions must stay aligned across package and Tauri metadata.", [
    `root package.json: ${String(rootPackage.version)}`,
    `apps/desktop/package.json: ${String(desktopPackage.version)}`,
    `apps/desktop/src-tauri/tauri.conf.json: ${String(tauriConfig.version)}`,
  ]);
}

const missingRootScripts = requiredRootScripts.filter((scriptName) => !rootPackage.scripts?.[scriptName]);
if (missingRootScripts.length > 0) {
  reportFailure("Root package.json is missing release validation scripts.", missingRootScripts);
}

const missingDesktopScripts = requiredDesktopScripts.filter((scriptName) => !desktopPackage.scripts?.[scriptName]);
if (missingDesktopScripts.length > 0) {
  reportFailure("Desktop package.json is missing release build scripts.", missingDesktopScripts);
}

if (!String(rootPackage.scripts.validate).includes("yarn release:check")) {
  reportFailure("Root validate script must include release readiness validation.", [
    "Expected `yarn release:check` inside package.json scripts.validate.",
  ]);
}

if (tauriConfig.productName !== "DevAtlas") {
  reportFailure("Tauri product name must remain DevAtlas.", [
    `Received ${String(tauriConfig.productName)}`,
  ]);
}

if (!tauriConfig.bundle?.active || tauriConfig.bundle?.targets !== "all") {
  reportFailure("Tauri bundle config must target all desktop release artifacts.", [
    "Expected bundle.active=true and bundle.targets=\"all\".",
  ]);
}

const postmanCommandNames = postmanCollection.item.map((item) => item.name).sort();
const typedCommandNames = extractTauriCommands(contractsSource).sort();
if (postmanCommandNames.join("\n") !== typedCommandNames.join("\n")) {
  reportFailure("Postman command names must match the typed Tauri command list.", [
    `Postman: ${postmanCommandNames.join(", ")}`,
    `TypeScript: ${typedCommandNames.join(", ")}`,
  ]);
}

const dockerRequirements = [
  ["Dockerfile", rootDockerfile, "cargo test --workspace"],
  ["Dockerfile", rootDockerfile, "yarn workspace @devatlas/desktop build"],
  ["apps/desktop/Dockerfile", desktopDockerfile, "yarn build"],
  ["apps/desktop/Dockerfile", desktopDockerfile, "cargo test --workspace"],
  ["docker-compose.yml", rootCompose, "devatlas-workspace"],
  ["apps/desktop/docker-compose.yml", desktopCompose, "desktop"],
];
const missingDockerSignals = dockerRequirements
  .filter(([_path, content, expectedText]) => !content.includes(expectedText))
  .map(([path, _content, expectedText]) => `${path} missing ${expectedText}`);
if (missingDockerSignals.length > 0) {
  reportFailure("Docker validation files are missing required release signals.", missingDockerSignals);
}

const workflowRequirements = [
  "pull_request:",
  "push:",
  "ubuntu-22.04",
  "actions/setup-node@v4",
  "dtolnay/rust-toolchain@stable",
  "yarn install --frozen-lockfile",
  "yarn validate",
];
const missingWorkflowSignals = workflowRequirements.filter((signal) => !githubWorkflow.includes(signal));
if (missingWorkflowSignals.length > 0) {
  reportFailure("GitHub Actions validation workflow is missing required release signals.", missingWorkflowSignals);
}

const checksumScriptRequirements = ["createHash(\"sha256\")", "sha256-checksums.txt", "checksums"];
const missingChecksumSignals = checksumScriptRequirements.filter((signal) => !checksumScript.includes(signal));
if (missingChecksumSignals.length > 0) {
  reportFailure("Release checksum generator is missing required SHA-256 behavior.", missingChecksumSignals);
}

const playbookRequirements = [
  "## Release Gate",
  "yarn validate",
  "## Version Alignment",
  "## Artifact Layout",
  "## Signing Boundary",
  "## Release Checklist",
  "## Release Notes Template",
  "## Known Issues",
  "Windows MSI",
  "macOS DMG",
  "Linux AppImage",
  "Checksums",
  "yarn release:checksums",
];
const missingPlaybookSignals = playbookRequirements.filter((signal) => !releasePlaybook.includes(signal));
if (missingPlaybookSignals.length > 0) {
  reportFailure("Release playbook is missing required release management sections.", missingPlaybookSignals);
}

const requiredReadmeSignals = [
  "## Docker",
  "docker compose up --build",
  "yarn release:check",
  "yarn release:checksums",
  "GitHub Actions",
  "docs/release-playbook.md",
];
const missingReadmeSignals = requiredReadmeSignals.filter((signal) => !readme.includes(signal));
if (missingReadmeSignals.length > 0) {
  reportFailure("README must document release validation workflow.", missingReadmeSignals);
}

console.log("release readiness ok");

function readText(relativePath) {
  return readFileSync(join(workspaceRoot, relativePath), "utf8");
}

function readJson(relativePath) {
  const absolutePath = join(workspaceRoot, relativePath);
  if (!existsSync(absolutePath)) {
    reportFailure("Required release file is missing.", [relativePath]);
  }
  return JSON.parse(readFileSync(absolutePath, "utf8"));
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

function reportFailure(message, details) {
  console.error(message);
  for (const detail of details) {
    console.error(`- ${detail}`);
  }
  process.exit(1);
}
