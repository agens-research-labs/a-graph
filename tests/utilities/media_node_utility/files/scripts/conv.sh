#!/bin/sh
in=$(cat)
path=$(echo "$in" | sed 's/.*"path":"\([^"]*\)".*/\1/')
mime=$(echo "$in" | sed -n 's/.*"mime_type":"\([^"]*\)".*/\1/p')
size=$(wc -c < "$path" | tr -d " ")
printf '{"text":"converted %s %s %s bytes"}' "$(basename "$path")" "$mime" "$size"
