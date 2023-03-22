LoadPackage("PatternClass");;
for i in [2..3] do
    for j in [2..7] do
		str := StringFormatted("./automatons/bns-{}-{}.tmp", i, j);;
        PrintTo(str, GraphToAut(BufferAndStack(i,j), 1, i + j + 2));;
    od;
od;
QuitGap();
