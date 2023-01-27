FROM rust:latest as builder
WORKDIR /usr/src/lsys-web-src
COPY . .
RUN cargo build -r




FROM debian:buster-slim
WORKDIR /usr/local/lsys-web
RUN apt-get update &&  apt-get install  -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/lsys-web-src/lsys-actix-web/.env /usr/local/lsys-web/.env
COPY --from=builder /usr/src/lsys-web-src/lsys-actix-web/static /usr/local/lsys-web/static
COPY --from=builder /usr/src/lsys-web-src/lsys-actix-web/config /usr/local/lsys-web/config
COPY --from=builder /usr/src/lsys-web-src/target/release/lsys-actix-web /usr/local/lsys-web/lsys-actix-web
COPY --from=builder /usr/src/lsys-web-src/.env /usr/local/lsys-web/.env
CMD ["/usr/local/lsys-web/lsys-actix-web"]