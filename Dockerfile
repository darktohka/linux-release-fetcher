FROM rust:alpine AS builder

WORKDIR /srv
COPY . .

RUN cargo build --profile release-lto

FROM scratch
COPY --from=builder /srv/target/release/linux-release-fetcher /linux-release-fetcher

ENTRYPOINT ["/linux-release-fetcher"]