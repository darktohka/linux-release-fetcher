FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /srv
COPY . .

RUN cargo build --profile release-lto

# Use a minimal image to run the application
FROM alpine:edge

RUN apk add --no-cache libssl3
COPY --from=builder /srv/target/release/linux-release-fetcher /srv/linux-release-fetcher

ENTRYPOINT ["/srv/linux-release-fetcher"]