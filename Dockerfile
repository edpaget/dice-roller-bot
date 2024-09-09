FROM scratch
COPY /target/release/discord-roller .
USER 1000
CMD ["./discord-roller"]
