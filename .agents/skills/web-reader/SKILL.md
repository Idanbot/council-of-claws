---
name: web-reader
description: Read the content of a specific URL. Returns clean markdown stripped of ads and navigation menus.
---

# web-reader

Use this skill when you have a specific URL (from a web search or user) and need to read its full content.

## Command Pattern

```bash
curl -s "https://r.jina.ai/{{url}}"
```

## Examples

Read a blog post:
```bash
curl -s "https://r.jina.ai/https://example.com/some-article"
```
