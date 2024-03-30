FROM debian:bullseye as lsys-builder
WORKDIR /tmp/lsys
COPY . /tmp/lsys
RUN apt-get update &&  apt-get install curl unzip git build-essential pkg-config wget libssl-dev -y && \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && . "$HOME/.cargo/env" && \
	wget https://nodejs.org/dist/v20.11.1/node-v20.11.1-linux-x64.tar.xz -O /tmp/node-v20.11.1-linux-x64.tar.xz && \
	mkdir -p /usr/local/lib/nodejs &&  tar -xJvf /tmp/node-v20.11.1-linux-x64.tar.xz -C /usr/local/lib/nodejs && \
	export PATH=/usr/local/lib/nodejs/node-v20.11.1-linux-x64/bin:$PATH && . ~/.profile && \
	./build.sh assets

FROM debian:bullseye-slim
WORKDIR /usr/local/lsys-web
COPY --from=lsys-builder /tmp/lsys/build/ /usr/local/lsys-web/
ENV \
	RUST_BACKTRACE=full \
    APP_HOST=0.0.0.0 \
    APP_PORT=80 \
    LOG_LEVEL=info \
    LOG_DIR="logs" \
    LOG_NAME="std::out" \
    DATABASE_URL="mysql://root:000@127.0.0.1:3306/test2" \
    REDIS_URL="redis://127.0.0.1/"
RUN apt-get update &&  apt-get install  -y libssl1.1 && apt-get clean && rm -rf /var/lib/apt/lists/*
CMD ["./lsys-actix-web"]