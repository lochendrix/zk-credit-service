FROM rust:1-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release -p zk_api
FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/zk_api /usr/local/bin/zk_api
CMD ["zk_api"]