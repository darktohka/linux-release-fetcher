FROM rust:alpine AS builder

WORKDIR /srv
COPY . .

RUN apk add perl clang g++ gcc && cargo build --profile release-lto && apk del perl clang g++ gcc

FROM scratch
COPY --from=builder /srv/target/release/linux-release-fetcher /linux-release-fetcher

ENTRYPOINT ["/linux-release-fetcher"]