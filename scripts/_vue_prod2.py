p = r"node_modules/vue/dist/vue.runtime.esm-browser.prod.js"
data = open(p, encoding="utf-8").read()
needle = ".then("
idx = 0
n = 0
while n < 40:
    i = data.find(needle, idx)
    if i < 0:
        break
    frag = data[i:i+80]
    if "value" in frag or "$" in frag:
        print(data[max(0,i-100):i+120])
        print("---")
        n += 1
    idx = i + 6
print("shown", n, "total search done")
