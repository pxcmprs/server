FROM ekidd/rust-musl-builder:stable AS builder
RUN USER=rust cargo init
COPY --chown=rust:rust Cargo.* ./
RUN cargo build --release
RUN rm -r target/x86_64-unknown-linux-musl/release/deps/pxcmprs_server*
COPY --chown=rust:rust src ./src
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/pxcmprs-server /usr/local/bin/

COPY Settings.toml ./

ENV PXCMPRS_SERVER__PORT 80
EXPOSE 80

ENTRYPOINT ["/usr/local/bin/pxcmprs-server"]