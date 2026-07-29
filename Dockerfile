FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
COPY src/ src/
RUN touch src/main.rs && cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/karesansui /usr/local/bin/karesansui
ENTRYPOINT ["karesansui"]
