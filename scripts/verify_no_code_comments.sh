#!/usr/bin/env bash
set -Eeuo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
mode="${1:-}"
if [[ "$mode" != '' && "$mode" != '--changed' ]]; then
    printf 'Usage: %s [--changed]\n' "$0" >&2
    exit 2
fi
if [[ "$mode" == '--changed' ]]; then
    if [[ "${GITHUB_EVENT_NAME:-}" == pull_request ]]; then
        base_ref="origin/${GITHUB_BASE_REF:-main}"
        mapfile -t candidates < <(git -C "$repo_root" diff --name-only --diff-filter=ACMRT "$base_ref...HEAD")
    else
        mapfile -t candidates < <(git -C "$repo_root" diff --name-only --diff-filter=ACMRT HEAD^ HEAD)
    fi
else
    mapfile -t candidates < <(git -C "$repo_root" ls-files)
fi
violations=()
for relative in "${candidates[@]}"; do
    case "$relative" in .git/*|target/*|vendor/*|node_modules/*) continue ;; esac
    case "${relative##*.}" in rs|toml|ps1|py|sh|yml|yaml) ;; *) continue ;; esac
    file="$repo_root/$relative"
    [[ -f "$file" ]] || continue
    if ! perl -0777 - "$file" "$relative" <<'PERL'
use strict;
use warnings;
my ($path, $label) = @ARGV;
open my $fh, '<:encoding(UTF-8)', $path or die "$path: $!";
local $/;
my $source = <$fh>;
my $line = 1;
my $index = 0;
my $length = $path =~ /\.rs\z/ ? length $source : 0;
my $state = 'normal';
my $raw_end = '';
my @found;
while ($index < $length) {
    my $tail = substr($source, $index);
    if ($state eq 'normal') {
        if ($tail =~ /\A(?:br|r)(#*)"/) {
            $raw_end = '"' . $1;
            $index += length $&;
            $state = 'raw';
            next;
        }
        if (substr($source, $index, 2) eq '//') {
            push @found, $line;
            my $end = index($source, "\n", $index);
            $index = $end < 0 ? $length : $end;
            next;
        }
        if (substr($source, $index, 2) eq '/*') {
            push @found, $line;
            my $depth = 1;
            $index += 2;
            while ($index < $length && $depth) {
                if (substr($source, $index, 2) eq '/*') {
                    $depth++;
                    $index += 2;
                } elsif (substr($source, $index, 2) eq '*/') {
                    $depth--;
                    $index += 2;
                } else {
                    $line++ if substr($source, $index, 1) eq "\n";
                    $index++;
                }
            }
            next;
        }
        my $character = substr($source, $index, 1);
        if ($character eq '"') {
            $state = 'string';
            $index++;
            next;
        }
        if ($character eq "'") {
            if ($tail =~ /\A'(?:\\.|[^'\\])'/) {
                $index += length $&;
            } else {
                $index++;
            }
            next;
        }
        $line++ if $character eq "\n";
        $index++;
    } elsif ($state eq 'raw') {
        my $end = index($source, $raw_end, $index);
        last if $end < 0;
        $line += (substr($source, $index, $end - $index) =~ tr/\n//);
        $index = $end + length $raw_end;
        $state = 'normal';
    } else {
        my $character = substr($source, $index, 1);
        if ($character eq '\\') {
            $index += 2;
        } elsif ($character eq '"') {
            $index++;
            $state = 'normal';
        } else {
            $line++ if $character eq "\n";
            $index++;
        }
    }
}
if ($path =~ /\.(?:toml|ps1|py|sh|yml|yaml)\z/) {
    my $number = 0;
    for my $text (split /\n/, $source, -1) {
        $number++;
        next if $number == 1 && $text =~ /^#!/;
        my $quote = '';
        for (my $i = 0; $i < length $text; $i++) {
            my $character = substr($text, $i, 1);
            if ($quote ne '') {
                if ($path =~ /\.ps1\z/ && $quote eq "'" && substr($text, $i, 2) eq "''") {
                    $i++;
                    next;
                }
                if ($path =~ /\.ps1\z/ && $quote eq '"' && $character eq chr(96)) {
                    $i++;
                    next;
                }
                if ($character eq $quote && ($path =~ /\.ps1\z/ || $i == 0 || substr($text, $i - 1, 1) ne '\\')) {
                    $quote = '';
                }
            } elsif ($character eq "'" || $character eq '"') {
                $quote = $character;
            } elsif ($character eq '#' && ($i == 0 || substr($text, $i - 1, 1) =~ /\s/)) {
                push @found, $number;
                last;
            }
        }
    }
}
if (@found) {
    print "$label:$_\n" for @found;
    exit 1;
}
PERL
    then
        violations+=("$relative")
    fi
done
if (("${#violations[@]}" > 0)); then
    printf '%s\n' 'Code comments are forbidden:'
    printf '%s\n' "${violations[@]}"
    exit 1
fi
printf '%s\n' 'Comment policy verified.'
