# OpenClaw Gateway Config (Bootstrap)

This directory contains the repo-managed OpenClaw bootstrap config.

## Files

- `openclaw.json5`: the repo-managed template copied into the writable OpenClaw state directory on first boot.
- Repo-local OpenClaw skills live in `.agents/skills` at the repository root, not in this folder.

## Image Path

The gateway image build copies this directory into:

- `/opt/council/bootstrap-config`

## Notes

- The gateway image bakes this template under `/opt/council/bootstrap-config`, and the compose command copies it into `data/openclaw/config/openclaw.json5` only if the runtime file does not already exist.
- OpenClaw reads repo-managed skills from `/repo/.agents/skills` via `skills.load.extraDirs`.
- On first boot, OpenClaw may rewrite the copied file in `data/openclaw/config/openclaw.json5` to add runtime-managed `channels` and `meta` fields.
- After that first copy, runtime changes to `data/openclaw/config/openclaw.json5` persist across container restarts.
- The tradeoff is that later edits to the repo bootstrap file do not automatically replace an existing runtime file. To re-seed from the repo version, remove `data/openclaw/config/openclaw.json5` and restart the gateway.
- Gateway auth stays enabled. Open the Control UI through `http://127.0.0.1:3000/gateway` or a raw tokenized URL.
