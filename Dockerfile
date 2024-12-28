FROM rust:alpine AS builder

WORKDIR /srv
COPY . .

RUN apk add musl-dev && cargo build --profile release-lto

FROM scratch
COPY --from=builder /srv/target/release-lto/linux-release-fetcher /linux-release-fetcher

ENTRYPOINT ["/linux-release-fetcher"]