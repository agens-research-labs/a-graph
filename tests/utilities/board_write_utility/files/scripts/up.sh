#!/bin/sh
text=$(cat | sed 's/.*"text":"\([^"]*\)".*/\1/' | tr a-z A-Z)
printf '{"text":"%s"}' "$text"
