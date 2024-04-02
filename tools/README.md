# Tools for analysing the Mac ROM

I've built some custom tools to pull stuff apart/adjust it:

 * `extract_traps` decodes the compressed trap table in ROM and
   matches it against the known names of OS and toolbox functions to
   produce a Ghidra script that will label all the trap functions in
   the ROM.
 * `extract_edisks` extracts disk images that match the encoding
   format the Mac Classic ROM image expects.
