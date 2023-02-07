.PHONY: c rd run dbg dbg-fmt lib err

default:
	cargo check

vk.xml:
	echo "Please put a valid xk.xml file in the root of the generator"
	false

/tmp/tmp.rs: vk.xml
	cargo run --bin vk_stdout vk.xml > /tmp/tmp.rs
	rustfmt /tmp/tmp.rs

rd: /tmp/tmp.rs
	vim /tmp/tmp.rs

/tmp/tmp: /tmp/tmp.rs
	rustc /tmp/tmp.rs 2> /tmp/out.txt --out-dir /tmp

run: /tmp/tmp
	/tmp/tmp

cmp: /tmp/tmp.rs

dbg:
	cargo run --bin vk_stdout vk.xml 2>&1 | less

dbg-fmt:
	rustfmt /tmp/tmp.rs 2>&1 | less

err:
	vim /tmp/out.txt

lib:
	cargo run --bin gen_lib_cmd vk.xml /tmp/test/pvk/src

c:
	rm -f /tmp/tmp
	rm -f /tmp/tmp.rs