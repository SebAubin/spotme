FROM rust:1-stretch as rust-build

WORKDIR /rust

COPY . .

RUN cargo build --release

FROM debian:stretch-slim

WORKDIR /etc/spotme

COPY --from=rust-build /rust/target/release/spotme .

EXPOSE 7974

ENTRYPOINT [ "./spotme" ]