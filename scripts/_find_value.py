import os
root = "node_modules"
hits = []
for dirpath, dirs, files in os.walk(root):
    if any(part in ("@vue", "vue", "nuxt", "@nuxt", "@intlify", "pinia", "@vueuse", "@tauri-apps") for part in dirpath.split(os.sep)):
        pass
    else:
        # skip unrelated top-level packages after first level
        rel = os.path.relpath(dirpath, root)
        top = rel.split(os.sep)[0]
        if top not in (".", "@vue", "vue", "nuxt", "@nuxt", "@intlify", "pinia", "@vueuse", "@tauri-apps", "@nuxtjs"):
            dirs.clear()
            continue
    for fn in files:
        if not fn.endswith((".js", ".mjs", ".cjs")):
            continue
        p = os.path.join(dirpath, fn)
        try:
            if os.path.getsize(p) > 6_000_000:
                continue
            data = open(p, "rb").read()
        except OSError:
            continue
        if b"$value" in data:
            hits.append(p)
print("\n".join(hits) if hits else "none")
