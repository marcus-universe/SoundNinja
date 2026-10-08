#!/usr/bin/env bash
# Detached ASCII signatures + SHA256SUMS for every file in dist-upload/.
set -euo pipefail

OUT="${OUT_DIR:-dist-upload}"
mkdir -p "$OUT"
cd "$OUT"

shopt -s nullglob
files=()
for f in *; do
  [[ -f "$f" ]] || continue
  case "$f" in
    *.asc|SHA256SUMS|SHA256SUMS.asc|soundninja-signing-key.asc) continue ;;
  esac
  files+=("$f")
done

if [[ ${#files[@]} -eq 0 ]]; then
  echo "::error::No assets to checksum in $OUT"
  exit 1
fi

sha256sum "${files[@]}" | sort -k2 > SHA256SUMS
echo "Wrote SHA256SUMS"

if [[ "${SIGN:-0}" != "1" ]]; then
  echo "SIGN!=1; skip gpg signatures"
  ls -la
  exit 0
fi

if [[ -n "${GPG_PUBLIC_KEY:-}" ]]; then
  printf '%s\n' "$GPG_PUBLIC_KEY" > soundninja-signing-key.asc
elif gpg --output soundninja-signing-key.asc --armor --export "${GPG_FINGERPRINT:-}" 2>/dev/null; then
  echo "Exported public key"
else
  echo "::warning::Could not export public key"
fi

for f in "${files[@]}" SHA256SUMS; do
  [[ -f "$f" ]] || continue
  rm -f "$f.asc"
  gpg --batch --yes --detach-sign --armor "$f"
  echo "Signed $f"
done

ls -la
