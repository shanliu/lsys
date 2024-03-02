#!/bin/bash

outputFile="tables.sql"
if [ -e "$outputFile" ]; then
    rm "$outputFile"
fi
for d in */; do
    for f in "$d"/*.sql; do
        if [ -e  "$f" ]; then
           cat "$f" >> "$outputFile"
           echo "" >> "$outputFile"
        fi
    done
	
done

