#!/usr/bin/env bash
set -Eeuo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
domain_dir="$repo_root/src/domain"
if [[ ! -d "$domain_dir" ]]; then
    printf 'Domain directory not found: %s\n' "$domain_dir" >&2
    exit 1
fi
forbidden_crates=(egui eframe wgpu wgpu_types winit raw_window_handle pollster bytemuck)
violations=()
while IFS= read -r -d '' file; do
    for crate in "${forbidden_crates[@]}"; do
        pattern="(use|extern[[:space:]]+crate)[[:space:]].*\\b${crate}\\b|\\b${crate}::"
        if grep -nE "$pattern" "$file" >/dev/null; then
            while IFS= read -r match; do
                violations+=("$file:$match")
            done < <(grep -nE "$pattern" "$file")
        fi
    done
done < <(find "$domain_dir" -type f -name '*.rs' -print0)
if (("${#violations[@]}" > 0)); then
    printf '%s\n' 'ARCHITECTURE VIOLATIONS DETECTED:'
    printf '  - %s\n' "${violations[@]}"
    exit 1
fi
printf '%s\n' 'Architecture OK: src/domain/ does not depend on infrastructure crates.'
