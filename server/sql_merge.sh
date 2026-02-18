#!/bin/bash

outputFile="tables.sql"
if [ -e "$outputFile" ]; then
    rm "$outputFile"
fi

# 递归查找所有子目录（包括examples）下的tables.sql文件
find . -type f -name "tables.sql" ! -path "./tables.sql" | sort | while read -r f; do
    if [ "$(uname)" = "Darwin" ]; then
        echo  "-- ---------- $f ------------" >> "$outputFile"
        cat "$f" >> "$outputFile"
        echo  "\n-- ---------- $f ------------" >> "$outputFile"
    else
        echo -e "-- ---------- $f ------------" >> "$outputFile"
        cat "$f" >> "$outputFile"
        echo -e  "\n-- ---------- $f ------------\n" >> "$outputFile"
    fi
done
