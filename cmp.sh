#!/bin/bash

file1="/tmp/tmp.rs"
file2="/tmp/old_tmp.rs"

s1="/tmp/tmp_sorted.rs"
s2="/tmp/old_tmp_sorted.rs"

sort "$file1" > "$s1"
sort "$file2" > "$s2"

if cmp -s "$s1" "$s2"; then
    printf 'The file "%s" is the SAME as "%s"\n' "$file1" "$file2"
else
    printf 'The file "%s" is DIFFERENT from "%s"\n' "$file1" "$file2"
fi