#!/bin/sh
# A tool that fails: writes why on the standard error and exits with 3.
cat > /dev/null
echo "boom" >&2
exit 3
