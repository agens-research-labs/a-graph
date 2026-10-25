#!/bin/sh
cat > /dev/null
echo "boom: cannot read the file" >&2
exit 3
