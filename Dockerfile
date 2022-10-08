FROM rust:latest as builder
WORKDIR /usr/src/reapi
COPY . .
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/reapi/target/release/reapi /usr/local/bin

EXPOSE 8080

CMD ["/usr/local/bin/reapi"]