FROM rust:1.87-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo generate-lockfile \
    && cargo build --release --locked

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --create-home appuser
WORKDIR /app
COPY --from=builder /app/target/release/app /usr/local/bin/sky-inventory
USER 10001:10001
EXPOSE 8080
ENV BIND_ADDR=0.0.0.0:8080
ENTRYPOINT ["/usr/local/bin/sky-inventory"]
