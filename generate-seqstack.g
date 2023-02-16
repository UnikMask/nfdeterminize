LoadPackage("PatternClass");;
str := "automatons/seqstack-3-3-3.tmp";;
PrintTo(str, GraphToAut(Seqstacks(3, 3, 3), 1, 11));;
QuitGap();
