#!/bin/sh
# A tool that takes too long.
cat > /dev/null
sleep 5
echo '{"late": true}'
