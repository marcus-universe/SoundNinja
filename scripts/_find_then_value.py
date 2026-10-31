import os
roots = [
    r"node_modules/@vue",
    r"node_modules/vue",
    r"node_modules/@intlify",
    r"node_modules/@nuxtjs",
    r"node_modules/nuxt/dist",
]
needle = b".then("
for root in roots:
    for dirpath, _, files in os.walk(root):
        for fn in files:
            if not fn.endswith((".js", ".mjs")):
                continue
            p = os.path.join(dirpath, fn)
            try:
                data = open(p, "rb").read()
            except OSError:
                continue
            if b"$value" in data or b"{value:" in data and b".then" in data:
                if b"$value" in data or b"value:e" in data or b"value: t" in data or b"value:e}" in data:
                    print("HIT", p)
