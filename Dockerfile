FROM rust:latest as rust_rust_builder
WORKDIR /usr/src/lsys-web
COPY ./server .
RUN cargo build -r && sed -i 's|../../../ui/public/|./ui/|g' ./usr/src/lsys-web/examples/lsys-actix-web/config/app.toml

FROM node:18-buster as node_builder
WORKDIR /usr/src/lsys-web
COPY ./ui .
RUN cd ./web && npm run build && cd .. && cd ./web && npm run build

FROM debian:buster-slim
WORKDIR /usr/local/lsys-web
RUN apt-get update &&  apt-get install  -y libssl1.1 && rm -rf /var/lib/apt/lists/*

ENV \
    APP_HOST=127.0.0.1 \
    APP_PORT=80 \
    #APP_SSL_PORT=443 
    LOG_LEVEL=sqlx_core=info,lsys_sender=debug,axtix_web=debug,actix=info,mio=error,lsys_user=trace,lsys_web=trace,lsys_core=trace,lsys_user=trace,lsys_rbac=trace,lsys_docs=trace,lsys_actix_web=trace,sqlx=trace,redis=debug \
    LOG_DIR="logs" \
    LOG_NAME="std::out" \
    #数据库配置 \
    DATABASE_URL="mysql://root:000@127.0.0.1:3306/test2" \
    #公共表前缀
    DATABASE_TABLE_PREFIX="yaf_" \
    #redis 配置
    REDIS_URL="redis://127.0.0.1/"

COPY --from=rust_builder /usr/src/lsys-web/examples/lsys-actix-web/static /usr/local/lsys-web/
COPY --from=rust_builder /usr/src/lsys-web/examples/lsys-actix-web/config /usr/local/lsys-web/
COPY --from=rust_builder /usr/src/lsys-web/examples/lsys-actix-web/locale /usr/local/lsys-web/
COPY --from=rust_builder /usr/src/lsys-web/examples/lsys-actix-web/data /usr/local/lsys-web/
COPY --from=rust_builder /usr/src/lsys-web/examples/lsys-actix-web/.env /usr/local/lsys-web/.env
COPY --from=rust_builder /usr/src/lsys-web/examples/target/release/lsys-actix-web /usr/local/lsys-web/lsys-actix-web
COPY --from=node_builder /usr/src/lsys-web/public /usr/local/lsys-web/ui
CMD ["/usr/local/lsys-web/lsys-actix-web"]