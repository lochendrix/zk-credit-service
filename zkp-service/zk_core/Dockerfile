FROM rust:1-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release -p zk_core
FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/zk_core /usr/local/bin/zk_core
CMD ["zk_core"]