---
name: obsidian-memory
description: Inspect mission and task summaries stored in the mounted Obsidian vault under /obsidian.
---

# obsidian-memory

Use this skill to review durable narrative memory written by the backend.

## Safety Rules

- Treat `/obsidian` as durable system state.
- Prefer read-only inspection with `find`, `rg`, and `sed -n`.
- Do not edit vault files unless the user explicitly asks for it.

## Preferred Commands

Find matching notes:

```bash
find /obsidian -type f | sort
rg -n "mission-123|task-456" /obsidian
```

Read a note:

```bash
sed -n '1,220p' /obsidian/path/to/note.md
```
