FROM rust:1 AS build
WORKDIR /usr/src

RUN apt-get update
RUN apt-get install musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl
RUN USER=root cargo new dice_roller
WORKDIR /usr/src/dice_roller
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=build /usr/local/cargo/bin/discord-roller .
USER 1000
CMD ["./discord-roller"]
