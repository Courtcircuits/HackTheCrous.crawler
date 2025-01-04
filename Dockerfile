ARG RUST_VERSION=1.77
FROM rust:${RUST_VERSION}-buster AS dependency
WORKDIR /opt/hackthecrous

RUN mkdir -p src && echo "fn main() {}" >> src/main.rs

COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo fetch

FROM dependency AS build

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools

COPY src src
COPY migrations migrations
RUN --mount=type=cache,target=/opt/target/ \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml  \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock  \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --target=x86_64-unknown-linux-musl --release && \
    cp ./target/x86_64-unknown-linux-musl/release/HackTheCrous-crawler /bin/crawler

FROM debian:bullseye-slim AS final

# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
RUN apt-get update && apt-get install --no-install-recommends -y ca-certificates && apt-get clean -- rm -rf /var/lib/apt/lists/*
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "1000" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/crawler /bin/
COPY migrations /bin/migrations

# What the container should run when it is started.
ENTRYPOINT ["/bin/bash", "-c"]
