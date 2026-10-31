p = r"node_modules/vue/dist/vue.runtime.esm-browser.prod.js"
data = open(p, encoding="utf-8").read()
for needle in ["$value", "value:e", "{value:", ".then(e=>"]:
    i = 0
    c = 0
    print("NEEDLE", needle)
    while c < 6:
        j = data.find(needle, i)
        if j < 0:
            break
        print(data[max(0, j - 120): j + 160].replace("\n", " "))
        print("---")
        c += 1
        i = j + len(needle)
    print("count", c)
