#!/bin/sh
# Serves the directory that contains index.html (this script’s folder). Opens http://127.0.0.1:$PORT/ in the default browser.

root=$(CDPATH= cd -- "$(dirname "$0")" && pwd) || exit 1
cd -- "$root" || exit 1

PORT=${PORT:-8000}
URL="http://127.0.0.1:${PORT}/"

cleanup() {
	if [ -n "$srv_pid" ]; then
		kill "$srv_pid" 2>/dev/null || true
	fi
}
trap cleanup INT TERM EXIT

python3 -m http.server "$PORT" --directory . &
srv_pid=$!

python3 -c "
import socket, sys, time
port = int(sys.argv[1])
for _ in range(80):
    try:
        s = socket.create_connection(('127.0.0.1', port), timeout=0.1)
        s.close()
        sys.exit(0)
    except OSError:
        time.sleep(0.05)
sys.exit(1)
" "$PORT" || true

python3 -c "import webbrowser; webbrowser.open(sys.argv[1])" "$URL" 2>/dev/null || true

wait "$srv_pid"
