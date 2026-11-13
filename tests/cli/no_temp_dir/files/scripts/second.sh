#!/bin/sh
in=$(cat)
data=$(echo "$in" | sed -n 's/.*"data":"\([^"]*\)".*/\1/p')
mime=$(echo "$in" | sed -n 's/.*"mime_type":"\([^"]*\)".*/\1/p')
path=$(echo "$in" | sed -n 's/.*"path":"\([^"]*\)".*/\1/p')
printf '{"text":"data [%s], type [%s], path [%s]"}' "$data" "$mime" "$path"
