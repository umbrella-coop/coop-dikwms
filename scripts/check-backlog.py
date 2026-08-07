#!/usr/bin/env python3
"""Validate docs/specs/backlog.md structure (SPEC intake queue).

Fails if:
- a `## SPEC-NNN` header has no following requirement block
- a requirement body exists without a preceding header
- headers are duplicated
- the header count differs from the requirement-block count

Run after every backlog edit (see AGENTS.md — Spec Intake Rule).
"""
import re
import sys
from pathlib import Path

path = Path(__file__).parent.parent / "docs" / "specs" / "backlog.md"
lines = path.read_text().splitlines()

errors = []
headers = []
reqs = []
current = None
in_requirement = False

for i, line in enumerate(lines, 1):
    if line.startswith("## SPEC-"):
        headers.append((i, line))
        current = line
        in_requirement = False
    elif line.startswith("**Requirement (captured"):
        reqs.append((i, line))
        if current is None:
            errors.append(f"line {i}: requirement block without a preceding header")
        else:
            in_requirement = True
    elif line.startswith("## ") and not line.startswith("## SPEC-"):
        current = None
        in_requirement = False

# header -> has requirement?
header_lines = {h for _, h in headers}
seen = set()
for i, h in headers:
    if h in seen:
        errors.append(f"line {i}: duplicate header {h}")
    seen.add(h)

# every header must have a requirement: walk pairs
for idx, (i, h) in enumerate(headers):
    next_header_line = headers[idx + 1][0] if idx + 1 < len(headers) else len(lines) + 1
    body = lines[i:next_header_line]
    if not any(l.startswith("**Requirement (captured") for l in body):
        errors.append(f"line {i}: header {h} has no requirement block")

if len(headers) != len(reqs):
    errors.append(
        f"header count ({len(headers)}) != requirement count ({len(reqs)})"
    )

if errors:
    print("BACKLOG STRUCTURE ERRORS:")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)

print(f"backlog ok: {len(headers)} entries, headers/requirements balanced")
