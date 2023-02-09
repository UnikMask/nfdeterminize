#!/bin/env gap.sh

LoadPackage("PatternClass");;
PrintTo("./automatons/bns-4-4.tmp", GraphToAut(BufferAndStack(4,4), 1, 10));;
QuitGap();;
