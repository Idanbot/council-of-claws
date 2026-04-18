# OpenClaw Gateway Config (Bootstrap)

This directory contains the repo-managed OpenClaw bootstrap config.

## Files

- `openclaw.json5`: the repo-managed template copied into the writable OpenClaw state directory on container start.
- Repo-local OpenClaw skills live in `.agents/skills` at the repository root, not in this folder.

## Image Path

The gateway image build copies this directory into:

- `/opt/council/bootstrap-config`

## Notes

- The gateway image bakes this template under `/opt/council/bootstrap-config`, and the compose command copies it into `data/openclaw/config/openclaw.json5` on container start.
- OpenClaw reads repo-managed skills from `/repo/.agents/skills` via `skills.load.extraDirs`.
- On first boot, OpenClaw may rewrite the copied file in `data/openclaw/config/openclaw.json5` to add runtime-managed `channels` and `meta` fields. The repo bootstrap file remains the human-edited source of truth.
- Gateway auth stays enabled. Open the Control UI through `http://127.0.0.1:3000/gateway` or a raw tokenized URL.
