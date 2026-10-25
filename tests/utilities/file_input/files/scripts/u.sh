#!/bin/sh
in=$(cat)
path=$(echo "$in" | sed 's/.*"path":"\([^"]*\)".*/\1/')
mime=$(echo "$in" | sed -n 's/.*"mime_type":"\([^"]*\)".*/\1/p')
printf '{"text":"%s|%s"}' "$path" "$mime"
