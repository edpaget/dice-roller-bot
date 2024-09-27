FROM alpine
COPY /target/x86_64-unknown-linux-musl/release/roller_discord .
USER 1000
CMD ["./roller_discord"]
