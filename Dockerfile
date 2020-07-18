FROM rust:1.45.0 AS build
WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl
RUN USER=root cargo new url-shortener
WORKDIR /usr/src/url-shortener
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=build /usr/local/cargo/bin/discord_dice_roller .
USER 1000
CMD ["./discord_dice_roller"]
