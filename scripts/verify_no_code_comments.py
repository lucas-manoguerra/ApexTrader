from pathlib import Path
import argparse
import io
import re
import subprocess
import sys
import tokenize

ROOT = Path(__file__).resolve().parents[1]
SKIP_DIRS = {".git", "target", "vendor", "node_modules"}
EXTENSIONS = {".rs", ".toml", ".ps1", ".py", ".sh", ".yml", ".yaml"}

def rust_comments(source):
    comments = []
    index = 0
    length = len(source)
    state = "normal"
    raw_end = ""
    while index < length:
        if state == "normal":
            raw = re.match(r"(?:br|r)(#*)\"", source[index:])
            if raw:
                raw_end = "\"" + raw.group(1)
                index += raw.end()
                state = "raw"
                continue
            if source.startswith("//", index):
                comments.append(source.count("\n", 0, index) + 1)
                end = source.find("\n", index)
                index = length if end < 0 else end
                continue
            if source.startswith("/*", index):
                comments.append(source.count("\n", 0, index) + 1)
                depth = 1
                index += 2
                while index < length and depth:
                    if source.startswith("/*", index):
                        depth += 1
                        index += 2
                    elif source.startswith("*/", index):
                        depth -= 1
                        index += 2
                    else:
                        index += 1
                continue
            if source[index] == '"':
                index += 1
                state = "string"
                continue
            if source[index] == "'":
                character = re.match(r"'(?:\\.|[^'\\])'", source[index:])
                if character:
                    index += character.end()
                else:
                    index += 1
                continue
            index += 1
        elif state == "raw":
            end = source.find(raw_end, index)
            if end < 0:
                break
            index = end + len(raw_end)
            state = "normal"
        else:
            if source[index] == "\\":
                index += 2
            elif source[index] == '"':
                index += 1
                state = "normal"
            else:
                index += 1
    return comments

def hash_comment_lines(source, path):
    found = []
    for number, line in enumerate(source.splitlines(), 1):
        quote = None
        index = 0
        while index < len(line):
            character = line[index]
            if quote:
                if path.suffix == ".ps1" and quote == "'" and line.startswith("''", index):
                    index += 2
                    continue
                if path.suffix == ".ps1" and quote == '"' and character == "`":
                    index += 2
                    continue
                if character == quote:
                    if path.suffix != ".ps1" and index > 0 and line[index - 1] == "\\":
                        index += 1
                        continue
                    quote = None
            elif character in {"'", '"'}:
                quote = character
            elif character == "#" and (index == 0 or line[index - 1].isspace()):
                found.append(number)
                break
            index += 1
    return found

def changed_paths():
    import os
    if os.environ.get("GITHUB_EVENT_NAME") == "pull_request":
        base_ref = "origin/" + os.environ.get("GITHUB_BASE_REF", "main")
        command = ["git", "diff", "--name-only", "--diff-filter=ACMRT", f"{base_ref}...HEAD"]
    else:
        command = ["git", "diff", "--name-only", "--diff-filter=ACMRT", "HEAD^", "HEAD"]
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return [ROOT / item for item in result.stdout.splitlines() if item]

def scan(path):
    source = path.read_text(encoding="utf-8")
    if path.suffix == ".rs":
        return rust_comments(source)
    if path.suffix == ".py":
        return [
            token.start[0]
            for token in tokenize.generate_tokens(io.StringIO(source).readline)
            if token.type == tokenize.COMMENT
        ]
    return hash_comment_lines(source, path)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--changed", action="store_true")
    args = parser.parse_args()
    candidates = changed_paths() if args.changed else list(ROOT.rglob("*"))
    violations = []
    for path in candidates:
        if not path.is_file() or path.suffix not in EXTENSIONS:
            continue
        try:
            relative = path.relative_to(ROOT)
        except ValueError:
            continue
        if any(part in SKIP_DIRS for part in relative.parts):
            continue
        for line in scan(path):
            violations.append(f"{relative}:{line}")
    if violations:
        print("Code comments are forbidden:")
        print("\n".join(violations))
        return 1
    print("Comment policy verified.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
