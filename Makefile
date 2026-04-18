SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help

PROJECT_ROOT := $(CURDIR)
BACKEND_DIR := $(PROJECT_ROOT)/apps/backend
DASHBOARD_DIR := $(PROJECT_ROOT)/apps/dashboard
COMPOSE_FILE := $(PROJECT_ROOT)/infra/compose/docker-compose.yml
COMPOSE_REMOTE_FILE := $(PROJECT_ROOT)/infra/compose/docker-compose.remote-worker.yml
ENV_FILE := $(PROJECT_ROOT)/.env
ENV_EXAMPLE := $(PROJECT_ROOT)/.env.example

COMPOSE_WITH_ENV := docker compose --env-file $(ENV_FILE) -f $(COMPOSE_FILE)

.PHONY: help setup setup-env deps install dashboard-install backend-fetch \
	check check-all lint format format-check ci \
	backend-check backend-test backend-fmt backend-fmt-check backend-clippy backend-run \
	dashboard-check dashboard-build dashboard-dev \
	compose-config compose-up compose-up-build compose-up-tunnel compose-up-build-tunnel compose-down compose-restart compose-ps compose-logs compose-pull gateway-url \
	remote-worker-up remote-worker-down \
	smoke smoke-compose smoke-gateway smoke-openclaw smoke-timezone smoke-internal-api smoke-agent-backend \
	preflight preflight-deploy post-start-verify \
	state-export state-import \
	clean prune

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*## "; printf "\nUsage:\n  make <target>\n\nTargets:\n"} /^[a-zA-Z0-9_.-]+:.*## / { printf "  %-22s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

setup: setup-env deps ## Bootstrap local project prerequisites

setup-env: ## Create .env from .env.example if missing
	@if [[ ! -f "$(ENV_FILE)" ]]; then \
		cp "$(ENV_EXAMPLE)" "$(ENV_FILE)"; \
		echo "Created $(ENV_FILE) from $(ENV_EXAMPLE)"; \
	else \
		echo "$(ENV_FILE) already exists"; \
	fi

deps: install ## Install all local dependencies

install: dashboard-install backend-fetch ## Install dashboard npm deps and prefetch backend crates

dashboard-install: ## Install dashboard dependencies
	npm --prefix "$(DASHBOARD_DIR)" install --no-audit --no-fund

backend-fetch: ## Fetch backend Cargo dependencies
	cargo fetch --manifest-path "$(BACKEND_DIR)/Cargo.toml"

check: backend-check dashboard-check compose-config ## Run core checks (backend, dashboard, compose)

check-all: format-check lint smoke ## Run full quality gate and smoke checks

lint: backend-clippy dashboard-check ## Run lint-like checks

format: backend-fmt ## Apply formatting

format-check: backend-fmt-check ## Verify formatting without changing files

ci: check-all ## Alias for CI-oriented full check target

backend-check: ## Compile-check backend
	cargo check --manifest-path "$(BACKEND_DIR)/Cargo.toml"

backend-test: ## Run backend tests
	cargo test --manifest-path "$(BACKEND_DIR)/Cargo.toml"

backend-fmt: ## Format backend Rust sources
	cargo fmt --manifest-path "$(BACKEND_DIR)/Cargo.toml"

backend-fmt-check: ## Check backend formatting
	cargo fmt --manifest-path "$(BACKEND_DIR)/Cargo.toml" -- --check

backend-clippy: ## Run backend clippy checks
	cargo clippy --manifest-path "$(BACKEND_DIR)/Cargo.toml" --all-targets --all-features -- -D warnings

backend-run: ## Run backend service locally
	cargo run --manifest-path "$(BACKEND_DIR)/Cargo.toml"

dashboard-check: ## Run dashboard type/svelte checks
	npm --prefix "$(DASHBOARD_DIR)" run check

dashboard-build: ## Build dashboard production bundle
	npm --prefix "$(DASHBOARD_DIR)" run build

dashboard-dev: ## Run dashboard dev server
	npm --prefix "$(DASHBOARD_DIR)" run dev

compose-config: ## Validate docker-compose config
	$(COMPOSE_WITH_ENV) config -q && echo compose-ok

compose-up: ## Start full stack (without build)
	bash "$(PROJECT_ROOT)/scripts/dev/prepare-data-dirs.sh"
	$(COMPOSE_WITH_ENV) up -d

compose-up-build: ## Build and start full stack
	bash "$(PROJECT_ROOT)/scripts/dev/prepare-data-dirs.sh"
	$(COMPOSE_WITH_ENV) up -d --build

compose-up-tunnel: ## Start full stack with optional cloudflared tunnel profile
	bash "$(PROJECT_ROOT)/scripts/dev/prepare-data-dirs.sh"
	$(COMPOSE_WITH_ENV) --profile tunnel up -d

compose-up-build-tunnel: ## Build and start full stack with optional cloudflared tunnel profile
	bash "$(PROJECT_ROOT)/scripts/dev/prepare-data-dirs.sh"
	$(COMPOSE_WITH_ENV) --profile tunnel up -d --build

compose-down: ## Stop full stack
	$(COMPOSE_WITH_ENV) down

compose-restart: ## Restart full stack services
	$(COMPOSE_WITH_ENV) restart

compose-ps: ## Show compose service status
	$(COMPOSE_WITH_ENV) ps

compose-logs: ## Tail compose logs
	$(COMPOSE_WITH_ENV) logs -f --tail=200

compose-pull: ## Pull compose images
	$(COMPOSE_WITH_ENV) pull

gateway-url: ## Print the tokenized OpenClaw Control UI URL
	@port="$$(grep -E '^GATEWAY_PORT=' "$(ENV_FILE)" | tail -n1 | cut -d= -f2-)"; \
	token="$$(grep -E '^OPENCLAW_GATEWAY_TOKEN=' "$(ENV_FILE)" | tail -n1 | cut -d= -f2-)"; \
	echo "http://127.0.0.1:$${port:-18789}/?token=$${token:-council-local-gateway-token}"

preflight: ## Verify local prerequisites and ports before starting the stack
	bash "$(PROJECT_ROOT)/scripts/dev/preflight.sh"

preflight-deploy: ## Verify deploy-time prerequisites and fail on placeholder secrets
	bash "$(PROJECT_ROOT)/scripts/dev/preflight.sh" --deploy

post-start-verify: ## Verify dashboard, backend, OpenClaw, and authenticated coc-tool health after startup
	bash "$(PROJECT_ROOT)/scripts/dev/post-start-verify.sh"

remote-worker-up: ## Start experimental remote worker placeholder profile (not v1-supported)
	docker compose --env-file "$(ENV_FILE)" -f "$(COMPOSE_FILE)" -f "$(COMPOSE_REMOTE_FILE)" --profile remote-worker up -d worker

remote-worker-down: ## Stop experimental remote worker placeholder profile
	docker compose --env-file "$(ENV_FILE)" -f "$(COMPOSE_FILE)" -f "$(COMPOSE_REMOTE_FILE)" --profile remote-worker down

smoke: smoke-compose smoke-gateway smoke-openclaw smoke-timezone smoke-internal-api smoke-agent-backend ## Run all smoke tests

smoke-compose: ## Run compose smoke test
	bash "$(PROJECT_ROOT)/scripts/smoke/compose-smoke.sh"

smoke-gateway: ## Run gateway config smoke test
	bash "$(PROJECT_ROOT)/scripts/smoke/gateway-config-smoke.sh"

smoke-openclaw: ## Boot only Redis, Postgres, and OpenClaw and verify the tokenized gateway URL
	bash "$(PROJECT_ROOT)/scripts/smoke/openclaw-runtime-smoke.sh"

smoke-timezone: ## Verify timezone env propagation across services
	bash "$(PROJECT_ROOT)/scripts/smoke/timezone-smoke.sh"

smoke-internal-api: ## Validate internal task/mission endpoints with Obsidian output
	bash "$(PROJECT_ROOT)/scripts/smoke/internal-api-smoke.sh"

smoke-agent-backend: ## Validate authenticated agent-to-backend bridge
	bash "$(PROJECT_ROOT)/scripts/smoke/agent-backend-smoke.sh"

state-export: ## Export a sanitized data snapshot for resuming on another machine
	bash "$(PROJECT_ROOT)/scripts/dev/export-state.sh" "$(ARCHIVE)"

state-import: ## Import a previously exported state snapshot into ./data
	bash "$(PROJECT_ROOT)/scripts/dev/import-state.sh" "$(ARCHIVE)"

clean: ## Clean local build artifacts
	cargo clean --manifest-path "$(BACKEND_DIR)/Cargo.toml"
	rm -rf "$(DASHBOARD_DIR)/build" "$(DASHBOARD_DIR)/.svelte-kit"

prune: ## Aggressively prune docker resources used by this project
	bash "$(PROJECT_ROOT)/scripts/dev/prune.sh"
