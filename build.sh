#!/bin/bash

mkdir -p build 
cp  -fr ./README.MD ./build
cp  -fr ./LICENSE ./build
#
mkdir -p ./server/examples/lsys-actix-web/data
cd ./server/examples/lsys-actix-web  && cargo build \-r && cd ../../..
cp  -fr ./server/assets ./build
cp  -fr ./server/target/release/lsys-actix-web ./build
cp  -fr ./server/examples/lsys-actix-web/config ./build
cp  -fr ./server/examples/lsys-actix-web/data ./build
cp  -fr ./server/examples/lsys-actix-web/locale ./build
cp  -fr ./server/examples/lsys-actix-web/static ./build
cp  -fr ./server/examples/lsys-actix-web/.env ./build
#
cd ui/web && npm i  && npm run build && cd ../..
cd ui/wap && npm i  && npm run build && cd ../..
cp  -fr ./ui/public/ ./build/ui/
#
if [ "$(uname)" = "Darwin" ]; then
   sed -i '' "s|../../../ui/public/|./ui/|g" ./build/config/app.toml
else
   sed -i "s|../../../ui/public/|./ui/|g" ./build/config/app.toml
fi

cd ./build && tar -cvf ../lsys.tar.gz ./ && cd ..
#
echo -e "The compilation was successful, \
Please start the service with ( ./lsys-actix-web ) \
to start the service and access it via \033[1;32mhttp://127.0.0.1:8080\033[0m"