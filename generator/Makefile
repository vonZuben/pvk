
default:
	cargo check

/tmp/test_build.sh:
	cp test_build.sh /tmp
	chmod +x /tmp/test_build.sh

run: /tmp/test_build.sh
	cargo run --bin vk_stdout vk.xml > /tmp/tmp.rs
	rustfmt /tmp/tmp.rs
	vim /tmp/tmp.rs

dbg:
	cargo run 2>&1 | less

dbg-fmt:
	rustfmt /tmp/tmp.rs 2>&1 | less
