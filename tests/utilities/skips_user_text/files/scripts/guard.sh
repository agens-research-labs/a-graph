#!/bin/sh
# Fails when it gets the text of the user, which the node does not take; any other text comes back in upper case.
text=$(cat | sed 's/.*"text":"\([^"]*\)".*/\1/')
case "$text" in
  *user*) echo "got the text of the user" >&2; exit 1 ;;
esac
printf '{"text":"%s"}' "$(printf '%s' "$text" | tr a-z A-Z)"
