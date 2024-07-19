FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
# prepares the recipe, allows chef to cache
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# builds dependencies! (the caching docker layer)
RUN cargo chef cook --release --recipe-path recipe.json
# regular build of application, dependencies are all ready
COPY . .
RUN cargo build --release --bin app

# rust toolchain isn't needed!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/app /usr/local/bin

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/app"]
