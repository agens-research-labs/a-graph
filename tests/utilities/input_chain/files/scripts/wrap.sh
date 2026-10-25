#!/bin/sh
text=$(cat | sed 's/.*"text":"\([^"]*\)".*/\1/')
printf '{"text":"[%s]"}' "$text"
