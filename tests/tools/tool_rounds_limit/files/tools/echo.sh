#!/bin/sh
# A tool: answers {"got": <the arguments it received>}.
printf '{"got": %s}' "$(cat)"
