#!/bin/env sh

gap.sh "generate-bns.g" -q
for f in automatons/*.tmp; do
	if [ -f "$f" ]; then
		tr -d '\\\n' < "$f" > "${f%.*}.nfa"
		rm $f
	fi
done
