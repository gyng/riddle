#!/bin/sh
# Client for tools/driver.mjs:  tools/drive.sh <port> '<json op>'
[ $# -eq 2 ] || { echo "usage: tools/drive.sh <port> '{\"op\":\"text\"}'" >&2; exit 2; }
curl -s -X POST -H 'content-type: application/json' --data "$2" "http://127.0.0.1:$1/"
