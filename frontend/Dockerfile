FROM node:22-bookworm-slim AS build
RUN npm install -g pnpm@11.22.0
WORKDIR /app
COPY api-contract/ api-contract/
COPY frontend/ frontend/
RUN pnpm --dir api-contract install --frozen-lockfile && pnpm --dir api-contract build
RUN pnpm --dir frontend install --frozen-lockfile --filter @crowd/web && pnpm --dir frontend build
FROM caddy:2-alpine
COPY frontend/Caddyfile /etc/caddy/Caddyfile
COPY --from=build /app/frontend/apps/web/dist /srv
