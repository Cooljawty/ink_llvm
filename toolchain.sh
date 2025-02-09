#!/bin/bash

inkc=./target/debug/llvm-tutorial
wrapper_h="wrapper.h"
wrapper_src="wrapper.c"
exe=a.out

$inkc 2>&1 | llc -filetype=asm - | gcc -o $exe $wrapper_src -x assembler - && ./$exe
