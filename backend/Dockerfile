FROM rust:1.97.1-bookworm AS build
WORKDIR /app/backend
COPY backend/ ./
RUN cargo build --release --locked
FROM debian:bookworm-slim
# Match wekeeper: use USTC for Debian packages, including security updates.
RUN if [ -f /etc/apt/sources.list ]; then \
      sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g; s/security.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list; \
    fi \
    && if [ -f /etc/apt/sources.list.d/debian.sources ]; then \
      sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g; s/security.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources; \
    fi \
    && apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/backend/target/release/crowd-api /usr/local/bin/crowd-api
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["crowd-api"]
