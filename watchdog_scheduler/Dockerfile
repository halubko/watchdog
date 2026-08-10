FROM rust:1.96-slim AS builder

WORKDIR /app

COPY .sqlx .sqlx
ENV SQLX_OFFLINE=true

COPY Cargo.toml Cargo.lock ./
COPY watchdog_core/Cargo.toml watchdog_core/Cargo.toml
COPY watchdog_db/Cargo.toml watchdog_db/Cargo.toml
COPY watchdog_config/Cargo.toml watchdog_config/Cargo.toml
COPY watchdog_scheduler/Cargo.toml watchdog_scheduler/Cargo.toml
COPY watchdog_api/Cargo.toml watchdog_api/Cargo.toml

RUN mkdir -p watchdog_core/src watchdog_db/src watchdog_api/src watchdog_config/src watchdog_scheduler/src \
    && echo "fn main() {}" > watchdog_api/src/main.rs \
    && echo "// placeholder" > watchdog_core/src/lib.rs \
    && echo "// placeholder" > watchdog_db/src/lib.rs \
    && echo "// placeholder" > watchdog_scheduler/src/lib.rs \
    && echo "// placeholder" > watchdog_config/src/lib.rs \
    && echo "fn main() {}" > watchdog_scheduler/src/main.rs

RUN cargo build --release

COPY . .

RUN touch watchdog_api/src/main.rs \
    && touch watchdog_core/src/lib.rs \
    && touch watchdog_config/src/lib.rs \
    && touch watchdog_db/src/lib.rs \
    && touch watchdog_scheduler/src/main.rs \
    && cargo build --release --bin watchdog_scheduler

FROM debian:trixie-slim AS runner

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/watchdog_scheduler /app/watchdog_scheduler

CMD [ "./watchdog_scheduler" ]