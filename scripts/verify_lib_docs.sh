#!/usr/bin/env bash
set -Eeuo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
cargo_toml="$repo_root/Cargo.toml"
tasks_dir="$repo_root/odd/tasks"
printf '%s\n' '=== Library documentation verification — ApexTrader ==='
if [[ ! -f "$cargo_toml" ]]; then
    printf 'Cargo.toml not found: %s\n' "$cargo_toml" >&2
    exit 1
fi
declare -A dependencies=()
section=''
while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" =~ ^[[:space:]]*\[(dependencies|dev-dependencies|build-dependencies)(\.[^]]+)?\][[:space:]]*$ ]]; then
        section=dependencies
        continue
    fi
    if [[ "$line" =~ ^[[:space:]]*\[ ]]; then
        section=''
        continue
    fi
    [[ "$section" == dependencies ]] || continue
    [[ "$line" =~ ^[[:space:]]*(#|$) ]] && continue
    if [[ "$line" =~ ^[[:space:]]*([a-zA-Z0-9_-]+)[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
        dependencies["${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
    elif [[ "$line" =~ ^[[:space:]]*([a-zA-Z0-9_-]+)[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
        dependencies["${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
    fi
done < "$cargo_toml"
printf '%s\n' 'Dependencies declared in Cargo.toml:'
for dependency in "${!dependencies[@]}"; do
    printf '  %s = %s\n' "$dependency" "${dependencies[$dependency]}"
done
if [[ ! -d "$tasks_dir" ]]; then
    printf '%s\n' 'odd/tasks/ does not exist; no ODD tasks to verify.'
    exit 0
fi
status=0
while IFS= read -r -d '' task_file; do
    content="$(cat "$task_file")"
    mentions_dependency=0
    for dependency in "${!dependencies[@]}"; do
        if grep -Eiq "(^|[^[:alnum:]_-])${dependency//-/[-_]}([^[:alnum:]_-]|$)" <<< "$content"; then
            mentions_dependency=1
            break
        fi
    done
    if ((mentions_dependency)); then
        if grep -Eiq 'docs\.rs|documentación oficial|librerías consultadas|versión consultada|APIs usadas' <<< "$content"; then
            printf 'Task %s: OK — documentation references recorded.\n' "$(basename "$task_file" .md)"
        else
            printf 'Task %s mentions dependencies without recording documentation consulted.\n' "$(basename "$task_file" .md)" >&2
            status=1
        fi
    fi
done < <(find "$tasks_dir" -type f -name '*.md' -print0)
if ((status == 0)); then
    printf '\n%s\n' '=== Documentation verification completed ==='
else
    printf '\n%s\n' '=== INCONSISTENCIES FOUND ===' >&2
fi
exit "$status"
