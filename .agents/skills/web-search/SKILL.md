---
name: web-search
description: Search the web for real-time information, weather, news, or technical documentation. Returns AI-clean markdown results.
---

# web-search

Use this skill when you need to find information on the web that is not in your training data or the local repository.

## Command Pattern

```bash
curl -s "https://s.jina.ai/{{query}}"
```

## Examples

Search for current weather in Tel Aviv:
```bash
curl -s "https://s.jina.ai/current%20weather%20in%20Tel%20Aviv"
```

Search for latest Rust releases:
```bash
curl -s "https://s.jina.ai/latest%20rust%20programming%20language%20releases"
```
