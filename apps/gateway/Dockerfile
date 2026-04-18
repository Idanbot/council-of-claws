ARG OPENCLAW_BASE_IMAGE=ghcr.io/openclaw/openclaw@sha256:9d5f1dfbd5deedc37706c78f745b958ffbe9b4f20840cfb4d49c617a50326902
FROM ${OPENCLAW_BASE_IMAGE}

USER root
COPY config /opt/council/bootstrap-config
RUN chown -R node:node /opt/council/bootstrap-config
USER node
