#!/usr/bin/env bash

for f in automatons/*.tmp; do
	if [ -f "$f" ]; then
		tr -d '\\\n' < "$f" > "${f%.*}.nfa"
		rm $f
	fi
done
