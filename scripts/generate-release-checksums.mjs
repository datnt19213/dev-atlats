import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join, relative } from "node:path";

const artifactRoot = process.argv[2] ?? "artifacts";
const checksumDir = join(artifactRoot, "checksums");
const checksumFile = join(checksumDir, "sha256-checksums.txt");

if (!existsSync(artifactRoot) || !statSync(artifactRoot).isDirectory()) {
  reportFailure(`Artifact directory does not exist: ${artifactRoot}`);
}

mkdirSync(checksumDir, { recursive: true });

const artifactPaths = listFiles(artifactRoot)
  .filter((path) => !relative(artifactRoot, path).replaceAll("\\", "/").startsWith("checksums/"))
  .filter((path) => basename(path) !== "sha256-checksums.txt")
  .sort();

if (artifactPaths.length === 0) {
  reportFailure(`No release artifacts found in: ${artifactRoot}`);
}

const lines = artifactPaths.map((path) => {
  const hash = createHash("sha256").update(readFileSync(path)).digest("hex");
  const normalizedPath = relative(artifactRoot, path).replaceAll("\\", "/");
  return `${hash}  ${normalizedPath}`;
});

writeFileSync(checksumFile, `${lines.join("\n")}\n`, "utf8");
console.log(`wrote ${checksumFile}`);

function listFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = join(directory, entry.name);
    if (entry.isDirectory()) {
      return listFiles(entryPath);
    }
    if (entry.isFile()) {
      return [entryPath];
    }
    return [];
  });
}

function reportFailure(message) {
  console.error(message);
  process.exit(1);
}
