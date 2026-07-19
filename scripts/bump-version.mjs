import { readFileSync, writeFileSync } from "fs";

const releaseType = process.argv[2];

if (!["major", "minor", "patch"].includes(releaseType)) {
  console.error("Usage: node scripts/bump-version.mjs [major|minor|patch]");
  process.exit(1);
}

// Read and bump package.json
const pkgPath = "package.json";
const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
const [major, minor, patch] = pkg.version.split(".").map(Number);

let newVersion;
if (releaseType === "major") newVersion = `${major + 1}.0.0`;
else if (releaseType === "minor") newVersion = `${major}.${minor + 1}.0`;
else newVersion = `${major}.${minor}.${patch + 1}`;

console.log(`Bumping version: ${pkg.version} → ${newVersion}`);

pkg.version = newVersion;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
console.log("✓ package.json updated");

// Update tauri.conf.json
const tauriConfPath = "src-tauri/tauri.conf.json";
const tauriConf = JSON.parse(readFileSync(tauriConfPath, "utf8"));
tauriConf.version = newVersion;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");
console.log("✓ src-tauri/tauri.conf.json updated");

// Update Cargo.toml (only the [package] version, not dependency versions)
const cargoPath = "src-tauri/Cargo.toml";
let cargo = readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^(version\s*=\s*)"[\d.]+"(\s*$)/m, `$1"${newVersion}"$2`);
writeFileSync(cargoPath, cargo);
console.log("✓ src-tauri/Cargo.toml updated");

console.log(`\nVersion bumped to ${newVersion} ✓`);
