#!/bin/bash


printf "%25s\t%s\t%s\t%s\n" "File" "Ok" "Fail" "%"
for file in `ls test_results/*.csv`; do
	name=`basename $file`
	ok=`grep "OK" $file | wc -l`
	fail=`grep "FAIL" $file | wc -l`
	perc=0
	if [ $((ok + fail)) -gt 0 ]; then 
	perc=`awk "BEGIN {print $ok * 100 / ($ok + $fail)}" || expr 0`
	fi
	printf "%25s\t%02d\t%02d\t%.1f%%\n" $name $ok $fail $perc
done
