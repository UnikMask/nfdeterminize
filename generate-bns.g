LoadPackage("PatternClass");;
for i in [1..5] do
    for j in [1..4] do
		str := StringFormatted("./automatons/bns-{}-{}.tmp", i, j);;
        PrintTo(str, GraphToAut(BufferAndStack(i,j), 1, i + j + 2));;
    od;
od;
QuitGap();
