#!/bin/bash

#rustc tmp.rs 2>&1 | less
rustc tmp.rs 2> out.txt
vim out.txt
