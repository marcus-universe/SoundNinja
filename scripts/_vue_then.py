p = r"node_modules/@vue/runtime-core/dist/runtime-core.esm-bundler.js"
data = open(p, encoding="utf-8").read()
needle = ".then("
idx = 0
n = 0
while n < 30:
    i = data.find(needle, idx)
    if i < 0:
        break
    print("---", i)
    print(data[max(0, i - 180): i + 180].replace("\n", " "))
    print()
    n += 1
    idx = i + len(needle)
print("count shown", n)
