#!/bin/sh
cd "$(dirname "$0")" || exit 1

PORT=${PORT:-8000}
URL="http://127.0.0.1:${PORT}/"

(sleep 1; python3 -c "import sys, webbrowser; webbrowser.open(sys.argv[1])" "$URL" >/dev/null 2>&1) &

exec python3 -m http.server "$PORT" --directory .
