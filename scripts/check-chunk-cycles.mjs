// Fail the release build if the entry chunk can reach a static import cycle.
// A cycle that calls a binding at module top level throws (TDZ) and the
// window stays a blank background. Dynamic import() is not a cycle edge.
import { readdirSync, readFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'

const dist = join(process.cwd(), 'dist')
const nuxtDir = join(dist, '_nuxt')
const htmlPath = join(dist, 'index.html')

if (!existsSync(htmlPath) || !existsSync(nuxtDir)) {
  console.error('check-chunk-cycles: dist/index.html or dist/_nuxt is missing')
  process.exit(1)
}

const html = readFileSync(htmlPath, 'utf8')
const entryMatch = html.match(/<script[^>]*type="module"[^>]*src="[^"]*\/_nuxt\/([^"/]+\.js)"/)
  || html.match(/src="\/_nuxt\/([^"/]+\.js)"/)
const entry = entryMatch?.[1]
if (!entry) {
  console.error('check-chunk-cycles: no module entry in dist/index.html')
  process.exit(1)
}

const files = readdirSync(nuxtDir).filter((name) => name.endsWith('.js'))
const graph = new Map()
const importRe = /(?:import|export)[^'"\n]*?from\s*["']\.\/([^"']+\.js)["']|(?:import|export)\s*["']\.\/([^"']+\.js)["']/g

for (const name of files) {
  const source = readFileSync(join(nuxtDir, name), 'utf8')
  const deps = new Set()
  importRe.lastIndex = 0
  let match
  while ((match = importRe.exec(source))) {
    const dep = match[1] || match[2]
    if (dep) deps.add(dep)
  }
  graph.set(name, deps)
}

if (!graph.has(entry)) {
  console.error(`check-chunk-cycles: entry ${entry} is not in dist/_nuxt`)
  process.exit(1)
}

const WHITE = 0
const GRAY = 1
const BLACK = 2
const color = new Map()

function walk(name, stack) {
  color.set(name, GRAY)
  for (const dep of graph.get(name) || []) {
    const state = color.get(dep) || WHITE
    if (state === GRAY) {
      const at = stack.indexOf(dep)
      const cycle = stack.slice(at).concat(dep).join(' -> ')
      console.error(`check-chunk-cycles: static import cycle\n${cycle}`)
      process.exit(1)
    }
    if (state !== BLACK) walk(dep, stack.concat(name))
  }
  color.set(name, BLACK)
}

walk(entry, [])
console.log(`check-chunk-cycles: ${entry} has no static import cycle`)
