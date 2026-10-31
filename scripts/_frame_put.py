"""Upload a file to the Steam Frame. Does not print the password."""
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _frame_ssh import HOST, password

PSCP = r"C:\Program Files\PuTTY\pscp.exe"


def main() -> int:
    local, remote = sys.argv[1], sys.argv[2]
    result = subprocess.run(
        [PSCP, "-batch", "-pw", password(), local, f"{HOST}:{remote}"],
    )
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
