
default:
	cargo check

run:
	cargo run > /tmp/tmp.rs
	rustfmt /tmp/tmp.rs
	vim /tmp/tmp.rs

dbg:
	cargo run 2>&1 | less
