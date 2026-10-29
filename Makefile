.PHONY: docs-install docs-dev docs-build docs-preview install check check-web check-backend test-unit api-dev api-external dev-embedded test-embedded test-business web-dev miniapp-dev compose-up compose-down
install:
	pnpm --dir api-contract install --frozen-lockfile
	pnpm --dir frontend install --frozen-lockfile
check: check-web check-backend
check-web:
	pnpm --dir api-contract build
	pnpm --dir api-contract test
	pnpm --dir frontend check
check-backend:
	cargo fmt --manifest-path backend/Cargo.toml --all --check
	cargo check --manifest-path backend/Cargo.toml --locked
	cargo clippy --manifest-path backend/Cargo.toml --all-targets --locked -- -D warnings
	cargo test --manifest-path backend/Cargo.toml --locked
test-unit:
	cargo test --manifest-path backend/Cargo.toml --locked
api-dev: dev-embedded
api-external:
	cargo run --manifest-path backend/Cargo.toml --locked --bin crowd-api
dev-embedded:
	exec cargo run --locked --manifest-path backend/Cargo.toml --example embedded_postgres
test-embedded:
	cargo build --locked --manifest-path backend/Cargo.toml --example embedded_postgres
	python3 tools/check_embedded.py
test-business:
	exec cargo run --locked --manifest-path backend/Cargo.toml --example embedded_postgres -- --test-business
web-dev:
	pnpm --dir frontend dev
miniapp-dev:
	pnpm --dir frontend --filter @crowd/miniapp dev
DOCKER ?= docker
API_IMAGE ?= crowd-api:local
WEB_IMAGE ?= crowd-web:local
DOCKER_BUILD_FLAGS ?=
COMPOSE = $(DOCKER) compose --env-file .env -f compose.yaml -f compose.dev.yaml

.PHONY: docker-build docker-build-api docker-build-web docker-build-fresh compose-config compose-build compose-build-fresh
docker-build: docker-build-api docker-build-web
docker-build-fresh:
	$(MAKE) docker-build DOCKER_BUILD_FLAGS="--no-cache"
docker-build-api:
	$(DOCKER) build --pull $(DOCKER_BUILD_FLAGS) -f backend/Dockerfile -t "$(API_IMAGE)" .
docker-build-web:
	$(DOCKER) build --pull $(DOCKER_BUILD_FLAGS) -f frontend/Dockerfile -t "$(WEB_IMAGE)" .
compose-config:
	$(COMPOSE) config --quiet
compose-build:
	$(COMPOSE) build --pull
compose-build-fresh:
	$(COMPOSE) build --pull --no-cache
compose-up:
	docker compose --env-file .env -f compose.yaml -f compose.dev.yaml up --build --wait
compose-down:
	docker compose --env-file .env -f compose.yaml -f compose.dev.yaml down

docs-install:
	pnpm --dir docs install --frozen-lockfile
docs-dev:
	pnpm --dir docs dev
docs-build:
	pnpm --dir docs build
docs-preview:
	pnpm --dir docs preview

.PHONY: delivery-validate check-delivery
delivery-validate:
	python3 tools/check_delivery.py --validate-only
check-delivery:
	python3 tools/check_delivery.py
