#!/bin/sh
printf "args:"
for a in "$@"; do printf " [%s]" "$a"; done
printf "\ncwd: %s\n" "$(basename "$(pwd)")"
