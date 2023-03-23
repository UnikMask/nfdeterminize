#!/bin/env sh

GAP_PROG=$1

$GAP_PROG "generate-bns.g" -q
$GAP_PROG "generate-seqstack.g" -q
./clean-tmp-nfas.sh
