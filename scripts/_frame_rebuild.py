#!/usr/bin/env python3
"""Rebuild the Soundboard project from sqlite freelist rows (.recover)."""
import sqlite3
from pathlib import Path

RECOVER = Path("/tmp/sn-recover.sql")
SRC = Path("/home/steamos/Music/Soundboard/260912_Soundboard.sninja")
DST = Path("/tmp/sn-fixed.sninja")
PREFIX = "/home/steamos/Music/Soundboard/"


def to_linux(path: str) -> str:
    p = path.replace("\\", "/")
    key = "music/soundboard/"
    i = p.lower().find(key)
    if i >= 0:
        return PREFIX + p[i + len(key):]
    return p


def parse_values(raw: str):
    fields = []
    i = 0
    n = len(raw)
    while i < n:
        while i < n and raw[i] in " \t\r\n":
            i += 1
        if i >= n:
            break
        if raw[i] == "'":
            i += 1
            buf = []
            while i < n:
                if raw[i] == "'" and i + 1 < n and raw[i + 1] == "'":
                    buf.append("'")
                    i += 2
                    continue
                if raw[i] == "'":
                    i += 1
                    break
                buf.append(raw[i])
                i += 1
            fields.append("".join(buf))
        else:
            j = raw.find(",", i)
            token = (raw[i:] if j < 0 else raw[i:j]).strip()
            fields.append(None if token == "NULL" else token)
            if j < 0:
                break
            i = j
        while i < n and raw[i] in " ,":
            if raw[i] == ",":
                i += 1
                break
            i += 1
    return fields


def rows():
    text = RECOVER.read_text(errors="replace")
    marker = "INSERT INTO lost_and_found VALUES("
    idx = 0
    while True:
        i = text.find(marker, idx)
        if i < 0:
            return
        i += len(marker)
        # values end at the first `);` that is not inside a string
        j = i
        in_str = False
        while j < len(text):
            c = text[j]
            if in_str:
                if c == "'" and j + 1 < len(text) and text[j + 1] == "'":
                    j += 2
                    continue
                if c == "'":
                    in_str = False
            else:
                if c == "'":
                    in_str = True
                elif text.startswith(");", j):
                    yield parse_values(text[i:j])
                    idx = j + 2
                    break
            j += 1
        else:
            return


def num(v):
    if v is None or v == "":
        return None
    try:
        if isinstance(v, str) and "." in v:
            return float(v)
        return int(v)
    except ValueError:
        try:
            return float(v)
        except ValueError:
            return None


def main():
    sounds = {}  # linux path -> row
    links = {}  # (path, tab) -> index
    loose = []
    for f in rows():
        if len(f) < 5:
            continue
        nfield = f[2]
        if nfield == "14" and isinstance(f[4], str) and "Soundboard" in f[4]:
            path = to_linux(f[4])
            rid = num(f[3]) or 0
            prev = sounds.get(path)
            if prev and prev[0] >= rid:
                continue
            sounds[path] = (
                rid,
                path,
                f[5] if isinstance(f[5], str) else Path(path).stem,
                num(f[6]) if f[6] is not None else 1,
                f[7] if isinstance(f[7], str) else None,
                num(f[8]) or 0,
                1 if str(f[9]) == "1" else 0,
                f[10] if isinstance(f[10], str) else None,
                num(f[11]) if f[11] is not None else 50,
                num(f[12]) if f[12] is not None else 50,
                f[13] if isinstance(f[13], str) else None,
                num(f[14]),
                num(f[15]),
                num(f[16]),
                1 if str(f[17]) == "1" else 0,
            )
        elif nfield == "3" and isinstance(f[4], str) and "Soundboard" in f[4] and isinstance(f[5], str):
            path = to_linux(f[4])
            links[(path, f[5])] = num(f[6]) or 0
        elif nfield == "2" and isinstance(f[4], str) and "Soundboard" in f[4]:
            loose.append((to_linux(f[4]), num(f[5]) or 0))

    for path, index in loose:
        if path not in sounds:
            folder = path.rstrip("/").split("/")[-2] if path.count("/") >= 2 else ""
            sounds[path] = (
                0, path, Path(path).stem, 1, None, index, 1, None, 50, 50, None, None, None, None, 0,
            )
            links.setdefault((path, "All"), index)
            if folder and folder != "Soundboard":
                links.setdefault((path, folder), index)

    print("recovered sounds", len(sounds), "links", len(links))

    if DST.exists():
        DST.unlink()
    # Gifs and the 50 rows that still have media ids stay. Recover fills the rest.
    src = sqlite3.connect(f"file:{SRC}?mode=ro", uri=True)
    dst = sqlite3.connect(DST)
    src.backup(dst)
    src.close()
    dst.execute("PRAGMA journal_mode=DELETE")

    have = {row[0] for row in dst.execute("SELECT path FROM sounds")}
    added = 0
    for row in sounds.values():
        path = row[1]
        if path in have:
            continue
        dst.execute(
            """INSERT INTO sounds
               (path, name, volume, color, global_index, active, gif_id, gif_pos_x, gif_pos_y,
                sound_id, added_at, file_size, duration_secs, looping)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
            row[1:],
        )
        added += 1
        have.add(path)
    print("inserted sounds", added, "total", len(have))

    # Recovered membership wins. Keep a current link only when recover has none for that path.
    current = list(dst.execute("SELECT sound_path, tab, tab_index FROM sound_tabs"))
    recovered_paths = {p for (p, _t) in links}
    dst.execute("DELETE FROM sound_tabs")
    seen = set()
    def add_link(path, tab, index):
        key = (path, tab)
        if key in seen or path not in have:
            return
        seen.add(key)
        dst.execute(
            "INSERT INTO sound_tabs (sound_path, tab, tab_index) VALUES (?,?,?)",
            (path, tab, index or 0),
        )
    for (path, tab), index in links.items():
        add_link(path, tab, index)
    for path, tab, index in current:
        if path not in recovered_paths:
            add_link(path, tab, index)
    # Every sound belongs on All.
    for path, index in dst.execute("SELECT path, global_index FROM sounds"):
        add_link(path, "All", index or 0)

    have_tabs = {row[0] for row in dst.execute("SELECT name FROM tabs")}
    pos = max([row[0] for row in dst.execute("SELECT COALESCE(MAX(position), -1) FROM tabs")] + [-1])
    for (tab,) in dst.execute("SELECT DISTINCT tab FROM sound_tabs WHERE tab != 'All'"):
        if tab not in have_tabs:
            pos += 1
            dst.execute("INSERT INTO tabs (name, position) VALUES (?,?)", (tab, pos))
            have_tabs.add(tab)

    dst.commit()
    print("=== per tab ===")
    for tab, n in dst.execute(
        "SELECT tab, COUNT(*) AS n FROM sound_tabs WHERE tab != 'All' GROUP BY tab ORDER BY n DESC"
    ):
        print(f"{n:4} {tab}")
    missing = 0
    for (path,) in dst.execute("SELECT path FROM sounds"):
        if not Path(path).is_file():
            missing += 1
    print("sounds", dst.execute("SELECT COUNT(*) FROM sounds").fetchone()[0])
    print("missing files", missing)
    print("seps", dst.execute("SELECT COUNT(*) FROM separators").fetchone()[0])
    dst.close()


if __name__ == "__main__":
    main()
