---
name: repo-readonly
description: Inspect the Council repository from the read-only /repo mirror without editing files.
---

# repo-readonly

Use this skill to inspect code, docs, compose files, and scripts from inside OpenClaw.

## Safety Rules

- Treat `/repo` as read-only.
- Prefer `rg`, `find`, `sed -n`, `ls`, and `git diff --no-index` style inspection.
- Do not use destructive commands or write attempts under `/repo`.

## Preferred Commands

Find files:

```bash
rg --files /repo
```

Search for text:

```bash
rg -n "pattern" /repo
```

Read a file section:

```bash
sed -n '1,220p' /repo/path/to/file
```

List a subtree:

```bash
find /repo/path -maxdepth 2 -type f | sort
```
