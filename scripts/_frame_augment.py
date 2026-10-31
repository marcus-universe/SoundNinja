#!/usr/bin/env python3
"""Add on-disk audio that belongs to an existing soundboard folder tab."""
import sqlite3
from pathlib import Path

DB = Path("/tmp/sn-fixed.sninja")
ROOT = Path("/home/steamos/Music/Soundboard")
EXTS = {".wav", ".mp3", ".ogg", ".flac", ".m4a"}

db = sqlite3.connect(DB)
have = {row[0] for row in db.execute("SELECT path FROM sounds")}
tabs = {row[0] for row in db.execute("SELECT name FROM tabs")}
pos = db.execute("SELECT COALESCE(MAX(position), -1) FROM tabs").fetchone()[0]
added = 0
linked = 0
for folder in sorted(p for p in ROOT.iterdir() if p.is_dir()):
    if folder.name in ("Images",):
        continue
    files = [p for p in folder.iterdir() if p.is_file() and p.suffix.lower() in EXTS]
    if not files:
        continue
    if folder.name not in tabs:
        pos += 1
        db.execute("INSERT INTO tabs (name, position) VALUES (?,?)", (folder.name, pos))
        tabs.add(folder.name)
    for path in files:
        sp = str(path)
        if sp not in have:
            idx = db.execute("SELECT COALESCE(MAX(global_index), -1) FROM sounds").fetchone()[0] + 1
            db.execute(
                """INSERT INTO sounds
                   (path, name, volume, color, global_index, active, gif_pos_x, gif_pos_y, looping)
                   VALUES (?,?,1,NULL,?,1,50,50,0)""",
                (sp, path.stem, idx),
            )
            have.add(sp)
            added += 1
            db.execute(
                "INSERT INTO sound_tabs (sound_path, tab, tab_index) VALUES (?,?,?)",
                (sp, "All", idx),
            )
            db.execute(
                "INSERT INTO sound_tabs (sound_path, tab, tab_index) VALUES (?,?,?)",
                (sp, folder.name, idx),
            )
            linked += 1
            continue
        # File already in the project. Make sure its folder tab has it.
        row = db.execute(
            "SELECT 1 FROM sound_tabs WHERE sound_path=? AND tab=?",
            (sp, folder.name),
        ).fetchone()
        if row:
            continue
        idx = db.execute("SELECT global_index FROM sounds WHERE path=?", (sp,)).fetchone()[0] or 0
        db.execute(
            "INSERT INTO sound_tabs (sound_path, tab, tab_index) VALUES (?,?,?)",
            (sp, folder.name, idx),
        )
        linked += 1

db.commit()
print("added", added, "linked", linked)
print("sounds", db.execute("SELECT COUNT(*) FROM sounds").fetchone()[0])
for tab, n in db.execute(
    "SELECT tab, COUNT(*) FROM sound_tabs WHERE tab!='All' GROUP BY tab ORDER BY 2 DESC"
):
    print(f"{n:4} {tab}")
missing = 0
for (p,) in db.execute("SELECT path FROM sounds"):
    if not Path(p).is_file():
        missing += 1
print("missing", missing)
db.close()
