#!/usr/bin/env bash
set -euo pipefail

# Public, generic hygiene checks only. Do not put real personal identifiers,
# live account IDs, private local paths, or real secret fragments in this
# tracked file. For local/private checks, create an untracked file at:
#   .atelier-local/hygiene-denylist.txt
# with one extended-regex pattern per line.

secret_key_prefix='sk-'
secret_project_prefix="${secret_key_prefix}proj-"
generic_patterns=(
  "${secret_key_prefix}[A-Za-z0-9_-]{20,}"
  "${secret_project_prefix}[A-Za-z0-9_-]{20,}"
  '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
  'telegram:[0-9:-]{6,}'
  'BEGIN (RSA |OPENSSH |EC |DSA |)PRIVATE KEY'
)

paths=(README.md AGENTS.md docs crates Cargo.toml .gitignore LICENSE)
existing=()
for path in "${paths[@]}"; do
  if [ -e "$path" ]; then
    existing+=("$path")
  fi
done

scan_pattern() {
  local pattern="$1"
  grep -RInE "$pattern" "${existing[@]}"
}

failed=0
for pattern in "${generic_patterns[@]}"; do
  if scan_pattern "$pattern"; then
    failed=1
  fi
done

local_denylist=".atelier-local/hygiene-denylist.txt"
if [ -f "$local_denylist" ]; then
  while IFS= read -r pattern; do
    case "$pattern" in
      ''|'#'*) continue ;;
    esac
    if scan_pattern "$pattern"; then
      failed=1
    fi
  done < "$local_denylist"
fi

if [ "$failed" -ne 0 ]; then
  echo 'Public hygiene scan found private identifiers or secret-like values.' >&2
  exit 1
fi

echo 'Public hygiene scan passed.'
