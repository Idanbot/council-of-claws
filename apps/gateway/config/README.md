# OpenClaw Gateway Config (Bootstrap)

This directory contains the repo-managed OpenClaw bootstrap config.

## Files

- `openclaw.json5`: the repo-managed template copied into the writable OpenClaw state directory on container start.
- Repo-local OpenClaw skills live in `.agents/skills` at the repository root, not in this folder.

## Mount Path

The compose stack mounts this directory read-only to:

- `/bootstrap-config`

## Notes

- The compose stack copies this template into `data/openclaw/config/openclaw.json5` and points `OPENCLAW_CONFIG_PATH` there instead of relying on `openclaw onboard`.
- OpenClaw reads repo-managed skills from `/repo/.agents/skills` via `skills.load.extraDirs`.
- On first boot, OpenClaw may rewrite the copied file in `data/openclaw/config/openclaw.json5` to add runtime-managed `channels` and `meta` fields. The repo bootstrap file remains the human-edited source of truth.
- Gateway auth stays enabled. Open the Control UI through `http://127.0.0.1:3000/gateway` or a raw tokenized URL.
