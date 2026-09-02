import { readFileSync, writeFileSync } from 'fs'
import { dirname, join } from 'path'
import { fileURLToPath } from 'url'

const releaseType = process.argv[2]

if (!['major', 'minor', 'patch'].includes(releaseType)) {
  console.error('Usage: npm run version:major|version:minor|version:patch')
  process.exit(1)
}

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const pkgPath = join(root, 'package.json')
const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'))
const [major, minor, patch] = pkg.version.split('.').map(Number)

let newVersion
if (releaseType === 'major') newVersion = `${major + 1}.0.0`
else if (releaseType === 'minor') newVersion = `${major}.${minor + 1}.0`
else newVersion = `${major}.${minor}.${patch + 1}`

console.log(`Bumping companion module: ${pkg.version} → ${newVersion}`)

pkg.version = newVersion
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n')
console.log('package.json updated')
console.log(`companion/manifest.json stays 0.0.0 — companion-module-build writes ${newVersion} at pack time`)
