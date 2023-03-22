LoadPackage("PatternClass");;
for i in [2..3] do
    for j in [2..5] do
		str := StringFormatted("./automatons/two_stack-{}-{}.tmp", i, j);;
        PrintTo(str, GraphToAut(Seqstacks(i,j), 1, i + j + 2));;
    od;
od;
QuitGap();
