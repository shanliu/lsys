#!/bin/bash

# 当cargo 中启用 data-sqlite-source 时,下载sqlite源码
if command -v apt-get >/dev/null 2>&1; then
   if command -v unzip >/dev/null 2>&1; then
        echo "unzip is install"
    else
        sudo apt install unzip
    fi
elif command -v yum >/dev/null 2>&1; then
   if command -v unzip >/dev/null 2>&1; then
        echo "unzip is install"
    else
        sudo yum install unzip
    fi
fi
if command -v unzip >/dev/null 2>&1; then
    echo "unzip is install"
else
    echo "unzip not be install"
    exit 1
fi
script_dir=`pwd`/$(dirname "$0")
echo "clear old download: $script_dir/sqlite-amalgamation" && rm -rf "$script_dir/sqlite-amalgamation*"
echo "download: https://github.com/shanliu/lsys/releases/download/v0.0.0/sqlite-amalgamation.zip"
wget https://github.com/shanliu/lsys/releases/download/v0.0.0/sqlite-amalgamation.zip -O $script_dir/sqlite-amalgamation.zip
echo "clear sqlite-amalgamation.zip"
unzip $script_dir/sqlite-amalgamation.zip -d $script_dir/sqlite-amalgamation
rm $script_dir/sqlite-amalgamation.zip
echo "download finish"