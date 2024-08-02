FROM rust:1.80 as builder

ADD . /app
WORKDIR /app
RUN cargo build --release --bin vtn

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends curl && apt-get clean

EXPOSE 3000
COPY --from=builder /app/target/release/vtn /opt/openadr/
WORKDIR /opt/openadr

ENTRYPOINT ["./vtn"]
