FROM lukemathwalker/cargo-chef:latest-rust-1.64 AS chef
WORKDIR /usr/src/grim-reaper

FROM chef AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=prepare /usr/src/grim-reaper/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM rust AS runtime
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=build /usr/src/grim-reaper/target/release/grim-reaper .
CMD ["./grim-reaper"]