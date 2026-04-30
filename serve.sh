#!/bin/sh
cd "$(dirname "$0")" || exit 1
exec python3 -m http.server 8000 --directory .
