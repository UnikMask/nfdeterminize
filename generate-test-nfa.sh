#!/bin/env sh

GAP_PROG=$1

$GAP_PROG "generate-bns.g" -q
./clean-tmp-nfas.sh
