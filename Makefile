.PHONY: install check check-web check-backend test-unit api-dev web-dev miniapp-dev compose-up compose-down
install:
	pnpm --dir api-contract install --frozen-lockfile
	pnpm --dir frontend install --frozen-lockfile
check: check-web check-backend
check-web:
	pnpm --dir api-contract build
	pnpm --dir frontend check
check-backend:
	cargo fmt --manifest-path backend/Cargo.toml --all --check
	cargo check --manifest-path backend/Cargo.toml --locked
	cargo clippy --manifest-path backend/Cargo.toml --all-targets --locked -- -D warnings
	cargo test --manifest-path backend/Cargo.toml --locked
test-unit:
	cargo test --manifest-path backend/Cargo.toml --locked
api-dev:
	cargo run --manifest-path backend/Cargo.toml --locked
web-dev:
	pnpm --dir frontend dev
miniapp-dev:
	pnpm --dir frontend --filter @crowd/miniapp dev
compose-up:
	docker compose --env-file .env -f compose.yaml -f compose.dev.yaml up --build --wait
compose-down:
	docker compose --env-file .env -f compose.yaml -f compose.dev.yaml down
