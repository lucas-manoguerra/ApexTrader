#!/usr/bin/env bash
set -Eeuo pipefail
printf '%s\n' '=== Dependency audit — ApexTrader ==='
missing=()
for tool in cargo-audit cargo-deny; do
    command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
if (("${#missing[@]}" > 0)); then
    printf 'Missing required tools: %s\n' "${missing[*]}" >&2
    printf '%s\n' 'Install with: cargo install cargo-audit && cargo install cargo-deny' >&2
    exit 1
fi
printf '\n%s\n' '[1/2] Running cargo audit...'
cargo audit
printf '%s\n' 'cargo audit: no known vulnerabilities.'
printf '\n%s\n' '[2/2] Running cargo deny...'
cargo deny --config cargo-deny.toml --locked check
printf '%s\n' 'cargo deny: checks completed without issues.'
printf '\n%s\n' '=== Dependency audit completed ==='
