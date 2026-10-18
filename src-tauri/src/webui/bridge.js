/* SoundNinja remote mirror. Injected before the SPA so @tauri-apps/api
   talks to the desktop host over one websocket instead of IPC. */
(function () {
  if (window.__SN_REMOTE__) return
  var params = new URLSearchParams(location.search)
  var token = params.get('t') || ''
  try {
    if (token) localStorage.setItem('sn-webui-token', token)
    else token = localStorage.getItem('sn-webui-token') || ''
  } catch (e) { /* private mode */ }
  if (params.has('t')) {
    params.delete('t')
    var q = params.toString()
    history.replaceState(null, '', location.pathname + (q ? '?' + q : '') + location.hash)
  }
  window.__SN_REMOTE__ = true
  window.__SN_WEBUI_TOKEN__ = token
  document.documentElement.classList.add('sn-remote')
  try {
    var scale = localStorage.getItem('sn-remote-ui-scale')
    if (scale) document.documentElement.style.setProperty('--ui-scale', scale)
  } catch (e) { /* ignore */ }

  var callbacks = new Map()
  var listeners = new Map()
  var pending = new Map()
  var nextId = 1
  var ws = null
  var queue = []
  var stopped = false
  var attempt = 0
  var reconnectTimer = 0

  window.__SN_WEBUI_LOG__ = []
  window.__SN_WEBUI_STATE__ = 'connecting'

  function emitStatus(state, extra) {
    window.__SN_WEBUI_STATE__ = state
    var detail = { state: state }
    if (extra) detail.detail = extra
    window.dispatchEvent(new CustomEvent('sn-webui-status', { detail: detail }))
  }

  function dispatchApp(msg) {
    window.__SN_WEBUI_LOG__.push(msg)
    if (window.__SN_WEBUI_LOG__.length > 30) window.__SN_WEBUI_LOG__.shift()
    window.dispatchEvent(new CustomEvent('sn-webui', { detail: msg }))
  }

  function callListener(cbId, name, payload) {
    var cb = callbacks.get(cbId)
    if (typeof cb === 'function') cb({ event: name, id: cbId, payload: payload })
  }

  function dispatchEvent(name, payload) {
    listeners.forEach(function (entry) {
      if (entry.event === name) callListener(entry.cbId, name, payload)
    })
  }

  function send(obj) {
    var raw = JSON.stringify(obj)
    if (ws && ws.readyState === 1) ws.send(raw)
    else queue.push(raw)
  }
  window.__SN_WEBUI_SEND__ = send

  function rejectPending(err) {
    pending.forEach(function (p) { p.reject(err) })
    pending.clear()
  }

  function onMessage(raw) {
    var msg
    try { msg = JSON.parse(raw) } catch (e) { return }
    if (!msg || typeof msg !== 'object') return
    if (msg.type === 'event') {
      dispatchEvent(msg.name, msg.payload)
      return
    }
    if (msg.type === 'rpcResult') {
      var p = pending.get(msg.id)
      if (!p) return
      pending.delete(msg.id)
      if (msg.error) p.reject(new Error(String(msg.error)))
      else p.resolve(msg.result)
      return
    }
    if (msg.type === 'pong') return
    dispatchApp(msg)
  }

  function connect() {
    if (stopped) return
    emitStatus('connecting')
    var proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
    var url = proto + '//' + location.host + '/__sn/ws?t=' + encodeURIComponent(token)
    var sock
    try { sock = new WebSocket(url) } catch (e) {
      scheduleReconnect()
      return
    }
    ws = sock
    sock.onopen = function () {
      attempt = 0
      emitStatus('open')
      var pendingSend = queue
      queue = []
      for (var i = 0; i < pendingSend.length; i++) sock.send(pendingSend[i])
      try { sock.send(JSON.stringify({ type: 'hello', token: token })) } catch (e) { /* ignore */ }
    }
    sock.onmessage = function (ev) {
      if (typeof ev.data === 'string') onMessage(ev.data)
    }
    sock.onerror = function () { /* close follows */ }
    sock.onclose = function (ev) {
      ws = null
      rejectPending(new Error('connection closed'))
      if (ev && ev.code === 4403) {
        stopped = true
        emitStatus('rejected')
        return
      }
      emitStatus('closed')
      scheduleReconnect()
    }
  }

  function scheduleReconnect() {
    if (stopped) return
    var delay = Math.min(8000, 400 * Math.pow(2, attempt))
    attempt += 1
    clearTimeout(reconnectTimer)
    reconnectTimer = setTimeout(connect, delay)
  }

  function rpc(cmd, args) {
    var id = nextId++
    return new Promise(function (resolve, reject) {
      var timer = setTimeout(function () {
        if (!pending.has(id)) return
        pending.delete(id)
        reject(new Error('timed out'))
      }, 20000)
      pending.set(id, {
        resolve: function (v) { clearTimeout(timer); resolve(v) },
        reject: function (e) { clearTimeout(timer); reject(e) },
      })
      send({ type: 'rpc', id: id, cmd: cmd, args: args || {} })
    })
  }

  var BLOCKED = [
    'plugin:dialog|',
    'plugin:fs|',
    'plugin:updater|',
    'plugin:process|',
    'plugin:opener|',
    'plugin:global-shortcut|',
    'plugin:window|',
    'plugin:webview|',
  ]

  window.__TAURI_INTERNALS__ = {
    metadata: {
      currentWindow: { label: 'main' },
      currentWebview: { label: 'main', windowLabel: 'main' },
    },
    plugins: {},
    callbacks: callbacks,
    convertFileSrc: function (filePath) {
      var u = '/__sn/file?p=' + encodeURIComponent(filePath || '')
      if (token) u += '&t=' + encodeURIComponent(token)
      return u
    },
    transformCallback: function (callback, once) {
      var id = nextId++
      var wrapped = callback
      if (once) {
        wrapped = function (data) {
          callbacks.delete(id)
          if (callback) callback(data)
        }
      }
      callbacks.set(id, wrapped)
      return id
    },
    unregisterCallback: function (id) {
      callbacks.delete(id)
    },
    runCallback: function (id, data) {
      var cb = callbacks.get(id)
      if (typeof cb === 'function') cb(data)
    },
    invoke: function (cmd, args) {
      args = args || {}
      if (cmd === 'plugin:event|listen') {
        var eventId = nextId++
        listeners.set(eventId, { event: args.event, cbId: args.handler })
        return Promise.resolve(eventId)
      }
      if (cmd === 'plugin:event|unlisten') {
        var entry = listeners.get(args.eventId)
        listeners.delete(args.eventId)
        if (entry) callbacks.delete(entry.cbId)
        return Promise.resolve()
      }
      if (cmd === 'plugin:event|emit' || cmd === 'plugin:event|emit_to') {
        dispatchEvent(args.event, args.payload)
        return Promise.resolve()
      }
      if (cmd === 'plugin:sql|load') return Promise.resolve(args.db || 'remote')
      if (cmd === 'plugin:sql|select') return Promise.resolve([])
      if (cmd === 'plugin:sql|execute') return Promise.resolve({ rowsAffected: 0, lastInsertId: 0 })
      if (cmd === 'plugin:sql|close') return Promise.resolve(null)
      for (var i = 0; i < BLOCKED.length; i++) {
        if (cmd.indexOf(BLOCKED[i]) === 0) return Promise.resolve(null)
      }
      return rpc(cmd, args)
    },
  }

  function boot() {
    if (!token) {
      stopped = true
      emitStatus('rejected')
      return
    }
    fetch('/__sn/session?t=' + encodeURIComponent(token), { cache: 'no-store' })
      .then(function (res) {
        if (res.status === 403) {
          stopped = true
          emitStatus('rejected')
          return
        }
        connect()
      })
      .catch(function () { connect() })
  }
  boot()
})()
