#!/bin/bash

outputFile="tables.sql"
if [ -e "$outputFile" ]; then
    rm "$outputFile"
fi
for d in */; do
    for f in "$d"*.sql; do
        if [ -e  "$f" ]; then
            if [ "$(uname)" = "Darwin" ]; then
                echo  "-- ---------- $f ------------" >> "$outputFile"
                cat "$f" >> "$outputFile"
                echo  "\n-- ---------- $f ------------" >> "$outputFile"
            else
                echo -e "-- ---------- $f ------------" >> "$outputFile"
                cat "$f" >> "$outputFile"
                echo -e  "\n-- ---------- $f ------------\n" >> "$outputFile"
            fi
        fi
    done

done
