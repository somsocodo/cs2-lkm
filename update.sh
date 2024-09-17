#!/bin/bash
cd cs2-dumper
target/debug/cs2-dumper -f rs -o ../src/cs2_dumper
for file in ../src/cs2_dumper/*; do
    if [[ "$file" == *.so* ]]; then
        dir=$(dirname "$file")
        base=$(basename "$file")

        new_base="${base//.so/_so}"
        new_file="$dir/$new_base"

        mv "$file" "$new_file"
        echo "Renamed: $file to $new_file"
    fi
done