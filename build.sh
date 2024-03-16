#!/bin/bash
script_dir=`pwd`/$(dirname "$0")
echo "build on :$script_dir"
mkdir -p $script_dir/build
cp  -fr $script_dir/README.MD $script_dir/build
cp  -fr $script_dir/LICENSE $script_dir/build
#
mkdir -p $script_dir/server/examples/lsys-actix-web/data
cd $script_dir/server/examples/lsys-actix-web  
cargo build \-r 
cd $script_dir/server
bash $script_dir/server/sql_merge.sh 
cd $script_dir 
mkdir -p $script_dir/build/assets
cp  -fr $script_dir/server/tables.sql $script_dir/build/assets
cp  -fr $script_dir/server/target/release/lsys-actix-web $script_dir/build
cp  -fr $script_dir/server/examples/lsys-actix-web/config $script_dir/build
cp  -fr $script_dir/server/examples/lsys-actix-web/data $script_dir/build
cp  -fr $script_dir/server/examples/lsys-actix-web/locale $script_dir/build
cp  -fr $script_dir/server/examples/lsys-actix-web/static $script_dir/build
cp  -fr $script_dir/server/examples/lsys-actix-web/.env $script_dir/build
mkdir -p $script_dir/build/logs
#
cd $script_dir/ui/ 
npm i  && npm run build 
cd $script_dir/
cp -fr $script_dir/ui/public/ $script_dir/build/ui/
#
if [ "$(uname)" = "Darwin" ]; then
   sed -i '' "s|../../../ui/public/|./ui/|g" $script_dir/build/config/app.toml
else
   sed -i "s|../../../ui/public/|./ui/|g" $script_dir/build/config/app.toml
fi


has_assets=false
has_tar=false

for arg in "$@"
do
  case $arg in
    assets)
      has_assets=true
      ;;
    tar)
      has_tar=true
      ;;
  esac
done

if $has_assets; then
   curl -L -o "$script_dir/build/data/2023-7-area-code.csv.gz" "https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-code.csv.gz"
   curl -L -o "$script_dir/build/data/2023-7-area-geo.csv.gz" "https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-geo.csv.gz"
   curl -L -o "$script_dir/build/data/IP2LOCATION-LITE-DB11.BIN.zip" "https://github.com/shanliu/lsys/releases/download/v0.0.0/IP2LOCATION-LITE-DB11.BIN.zip"
   unzip -o "$script_dir/build/data/IP2LOCATION-LITE-DB11.BIN.zip" -d "$script_dir/build/data"
   rm -rf "$script_dir/build/data/IP2LOCATION-LITE-DB11.BIN.zip"
fi

if $has_tar; then
   cd $script_dir/build 
   tar -cvf $script_dir/lsys.tar.gz ./
   cd $script_dir
fi



echo -e "The compilation was successful, \
Please start the service with ( cd ./build && ./lsys-actix-web ) \
to the service and access it via \033[1;32mhttp://127.0.0.1:8080\033[0m"



