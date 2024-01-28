FROM rust:latest as builder
WORKDIR /usr/src/lsys-web-src
COPY . .
RUN cargo build -r




FROM debian:buster-slim
WORKDIR /usr/local/lsys-web
RUN apt-get update &&  apt-get install  -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/lsys-web-src/examples/lsys-actix-web/.env /usr/local/lsys-web/.env
COPY --from=builder /usr/src/lsys-web-src/examples/lsys-actix-web/static /usr/local/lsys-web/static
COPY --from=builder /usr/src/lsys-web-src/examples/lsys-actix-web/config /usr/local/lsys-web/config
COPY --from=builder /usr/src/lsys-web-src/examples/lsys-actix-web/locale /usr/local/lsys-web/locale
COPY --from=builder /usr/src/lsys-web-src/examples/lsys-actix-web/data /usr/local/lsys-web/data
COPY --from=builder /usr/src/lsys-web-src/examples/target/release/lsys-actix-web /usr/local/lsys-web/lsys-actix-web
CMD ["/usr/local/lsys-web/lsys-actix-web"]