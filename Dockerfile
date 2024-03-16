FROM debian:buster-slim as lsys-builder
WORKDIR /tmp/lsys
COPY . /tmp/lsys
RUN apt-get update &&  apt-get install curl unzip git build-essential pkg-config wget libssl-dev -y && \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && . "$HOME/.cargo/env" && \
	wget https://nodejs.org/dist/v20.11.1/node-v20.11.1-linux-x64.tar.xz -O /tmp/node-v20.11.1-linux-x64.tar.xz && \
	mkdir -p /usr/local/lib/nodejs &&  tar -xJvf /tmp/node-v20.11.1-linux-x64.tar.xz -C /usr/local/lib/nodejs && \
	export PATH=/usr/local/lib/nodejs/node-v20.11.1-linux-x64/bin:$PATH && . ~/.profile && \
	./build.sh assets

FROM debian:buster-slim as lsys-executor
RUN apt-get update &&  apt-get install  -y libssl1.1 && apt-get clean && rm -rf /var/lib/apt/lists/*

FROM lsys-executor
WORKDIR /usr/local/lsys-web
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

COPY --from=lsys-builder /tmp/lsys/build/ /usr/local/lsys-web/
CMD ["./lsys-actix-web"]