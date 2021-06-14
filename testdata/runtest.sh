#!/usr/bin/env bash
set -eu

ORIG=$1
IN=$(mktemp)
OUT=$(mktemp)

cleanup() {
	rm "$OUT"
	rm "$IN"
}
trap cleanup EXIT

{
	cat $ORIG
	dd if=/dev/urandom bs=128 count=1
} > $IN

PIPELINE=(
	filesrc location=$IN
	! image/jpeg,width=100,height=100
	! jpegtrunc
	! filesink location=$OUT
)

gst-launch-1.0 "${PIPELINE[@]}"

[[ $(stat -c%s "$OUT") -eq $(stat -c%s "$ORIG") ]] || {
	echo "eep"
	ls -l $ORIG $IN $OUT
	exit 1
}
