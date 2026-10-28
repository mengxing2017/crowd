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
