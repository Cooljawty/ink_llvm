#!/bin/bash

inkc=./target/debug/llvm-tutorial
llvm_path=./llvm-tools
llvm_as=$llvm_path/llvm-as
llvm_opt=$llvm_path/opt
llvm_llc=$llvm_path/llc

wrapper_h="wrapper.h"
wrapper_src="wrapper.c"
exe=a.out

compile_from_ll()
{
	$llvm_as $2
	$llvm_opt -O2 ${2%.ll}.bc \
		| $llvm_llc --relocation-model=pic - #PIC option used for linkage with GCC/ld 
}

link_from_asm()
{
	gcc -g -o $exe $wrapper_src -x assembler -
}

case $1 in
	install) install_llvm && compile_llvm ;;
	build)	compile_from_ll $@ ;;
	link)	compile_from_ll $@ | link_from_asm && ./$exe ;;
esac
