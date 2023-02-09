#!/bin/env sh

for f in automatons/*; do
	if [ -f "$f" ]; then
		tr -d '\\\n' < "$f" > "$f"
	fi
done
