---
name: human-messenger
description: Send a direct message to your human operator via Telegram. Use this when you are blocked, need clarification, or have a significant update.
---

# human-messenger

Use this skill when you need to reach out to the human in the loop.

## Safety Rules

- Be concise and clear.
- Do not spam the operator with frequent updates; prefer `log-report` for regular status.
- Use this when a task cannot proceed without human input.

## Command Pattern

```bash
/repo/scripts/coc-tool/coc-tool human-message --message "Your message here"
```

## Examples

Ask for clarification:
```bash
/repo/scripts/coc-tool/coc-tool human-message --message "I am blocked on task-123. The API key for Groq seems to be missing from the environment. Can you verify?"
```

Notify of a significant milestone:
```bash
/repo/scripts/coc-tool/coc-tool human-message --message "I have successfully migrated the production database. All smoke tests passed."
```
