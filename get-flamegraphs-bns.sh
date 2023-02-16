#!/bin/env sh

for i in {2..4..1}; do
	for j in {2..4..1}; do
		FLAMEGRAPH_PATH="flamegraphs/fg-bns-$i-$j.svg"
		cargo flamegraph -o $FLAMEGRAPH_PATH -- "automatons/bns-$i-$j.nfa"
	done
done
