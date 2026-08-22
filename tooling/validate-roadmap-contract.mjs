import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = dirname(dirname(fileURLToPath(import.meta.url)));

const roadmapStatus = JSON.parse(readText("docs/roadmap-status.json"));
const roadmapPlan = readText("plan/14-roadmap.md");
const cargoToml = readText("Cargo.toml");
const storageSource = readText("crates/storage-engine/src/lib.rs");

const requiredVersions = ["1.0", "1.5", "2.0", "2.5", "3.0", "3.5", "4.0", "4.5", "5.0"];
const futureRuntimeCrates = [
  "git-engine",
  "security-engine",
  "performance-engine",
  "plugin-engine",
  "mcp-server",
  "cloud-platform",
];

if (roadmapStatus.schemaVersion !== "1.0") {
  reportFailure("Roadmap status schema version must be 1.0.", [
    `Received ${String(roadmapStatus.schemaVersion)}`,
  ]);
}

if (roadmapStatus.currentTrack !== "1.0-mvp-foundation") {
  reportFailure("Roadmap current track must remain the MVP foundation.", [
    `Received ${String(roadmapStatus.currentTrack)}`,
  ]);
}

if (!roadmapStatus.runtimePolicy?.desktopFirst || !roadmapStatus.runtimePolicy?.localOnly) {
  reportFailure("Roadmap runtime policy must remain desktop-first and local-only.", [
    "runtimePolicy.desktopFirst and runtimePolicy.localOnly must both be true.",
  ]);
}

const trackVersions = new Set(roadmapStatus.tracks.map((track) => track.version));
const missingVersions = requiredVersions.filter((version) => !trackVersions.has(version));
if (missingVersions.length > 0) {
  reportFailure("Roadmap status is missing planned versions.", missingVersions);
}

for (const version of requiredVersions) {
  if (!roadmapPlan.includes(`# Version ${version}`)) {
    reportFailure("Roadmap source document is missing a required version heading.", [version]);
  }
}

const runtimeEnabledTracks = roadmapStatus.tracks.filter((track) => track.runtimeEnabled);
if (runtimeEnabledTracks.length !== 1 || runtimeEnabledTracks[0].version !== "1.0") {
  reportFailure("Only the active MVP foundation track may be runtime-enabled.", [
    `Runtime-enabled tracks: ${runtimeEnabledTracks.map((track) => track.version).join(", ")}`,
  ]);
}

const enabledFutureCrates = futureRuntimeCrates.filter((crateName) =>
  cargoToml.includes(`"crates/${crateName}"`),
);
if (enabledFutureCrates.length > 0) {
  reportFailure("Roadmap future crates must not be enabled in Cargo yet.", enabledFutureCrates);
}

if (/CREATE\s+TABLE\s+plugin_registry/i.test(storageSource)) {
  reportFailure("Roadmap future plugin storage must not be enabled in the current schema.", [
    "Found CREATE TABLE plugin_registry in crates/storage-engine/src/lib.rs.",
  ]);
}

console.log("roadmap contract ok");

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
