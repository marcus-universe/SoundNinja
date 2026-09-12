/**
 * Tauri WebView2 drops sockets. Node 23 then does
 * triggerUncaughtException(err, fromPromise) and nuxi restarts/exits.
 * Load via --import before nuxi so parent + fork both stay up.
 */
if (globalThis.__snEconnPatch) {
  // already applied
} else {
  globalThis.__snEconnPatch = true
  const flag = `--import ${import.meta.url}`
  if (!process.env.NODE_OPTIONS?.includes('patch-econnreset')) {
    process.env.NODE_OPTIONS = [process.env.NODE_OPTIONS, flag].filter(Boolean).join(' ')
  }

  const isTransient = (reason) => {
    const err = reason && typeof reason === 'object' ? reason : {}
    const msg = String(err.message || reason || '')
    return err.code === 'ECONNRESET' || err.code === 'EPIPE'
      || msg.includes('ECONNRESET') || msg.includes('EPIPE')
  }

  const rawEmit = process.emit.bind(process)
  process.emit = function patchedEmit(event, ...args) {
    if (
      (event === 'unhandledRejection' || event === 'uncaughtException')
      && isTransient(args[0])
    ) {
      // true = Node treats it as handled; do not forward to nuxi's once() killer
      return true
    }
    return rawEmit(event, ...args)
  }

  process.on('unhandledRejection', (reason) => {
    if (isTransient(reason)) return
  })
  process.on('uncaughtException', (err) => {
    if (isTransient(err)) return
    throw err
  })
}
