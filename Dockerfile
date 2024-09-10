FROM alpine
COPY /target/x86_64-unknown-linux-musl/release/discord-roller .
USER 1000
CMD ["./discord-roller"]
