FROM rustlang/rust:nightly-slim as planner
WORKDIR app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM rustlang/rust:nightly-slim as cacher
WORKDIR app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


FROM rustlang/rust:nightly-slim as builder
WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin termpad


FROM debian:buster-slim as runtime
WORKDIR app
COPY --from=builder /app/target/release/termpad /usr/local/bin
ENTRYPOINT ["/usr/local/bin/termpad"]
