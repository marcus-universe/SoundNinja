"""SSH helper. Reads Password from .env. Does not print it."""
import base64
import subprocess
import sys
from pathlib import Path

ENV = Path(__file__).resolve().parents[1] / ".env"
PLINK = r"C:\Program Files\PuTTY\plink.exe"
HOST = "steamos@192.168.50.170"


def password() -> str:
    for line in ENV.read_text(encoding="utf-8").splitlines():
        if line.startswith("Password"):
            return line.split("=", 1)[1].strip().strip('"')
    raise SystemExit("Password missing")


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "--get":
        remote = f"base64 -w 0 {sys.argv[2]}"
        result = subprocess.run([PLINK, "-ssh", "-batch", "-pw", password(), HOST, remote], capture_output=True)
        if result.returncode != 0:
            sys.stderr.buffer.write(result.stderr)
            return result.returncode
        Path(sys.argv[3]).write_bytes(base64.b64decode(result.stdout))
        print(f"saved {sys.argv[3]} {Path(sys.argv[3]).stat().st_size}")
        return 0
    if len(sys.argv) > 1 and sys.argv[1] == "--file":
        script = Path(sys.argv[2]).read_bytes()
        payload = base64.b64encode(script).decode("ascii")
        runner = "python3" if sys.argv[2].endswith(".py") else "bash"
        remote = f"echo {payload} | base64 -d | {runner}"
    else:
        remote = sys.argv[1] if len(sys.argv) > 1 else "uname -a"
    result = subprocess.run([PLINK, "-ssh", "-batch", "-pw", password(), HOST, remote], capture_output=True)
    sys.stdout.buffer.write(result.stdout)
    sys.stderr.buffer.write(result.stderr)
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
