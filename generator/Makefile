.PHONY: c rd run dbg dbg-fmt lib err

GEN_INPUT_FILES = vk.xml validusage.json

default:
	cargo check

$(GEN_INPUT_FILES):
	@echo "error: missing \""$@"\", please put in the root of the generator"
	@false

/tmp/tmp.rs: $(GEN_INPUT_FILES)
	cargo run --bin vk_stdout $(GEN_INPUT_FILES) > /tmp/tmp.rs
	rustfmt /tmp/tmp.rs

rd: /tmp/tmp.rs
	vim /tmp/tmp.rs

/tmp/tmp: /tmp/tmp.rs
	rustc /tmp/tmp.rs 2> /tmp/out.txt --out-dir /tmp

run: /tmp/tmp
	/tmp/tmp

cmp: /tmp/tmp.rs

dbg: $(GEN_INPUT_FILES)
	cargo run --bin vk_stdout $(GEN_INPUT_FILES) 2>&1 | less

dbg-fmt: /tmp/tmp.rs
	rustfmt /tmp/tmp.rs 2>&1 | less

err:
	vim /tmp/out.txt

lib: $(GEN_INPUT_FILES)
	cargo run --bin gen_lib_cmd $(GEN_INPUT_FILES) /tmp/test/pvk/src

c:
	@echo "cleaning tmp files"
	@rm -f /tmp/tmp
	@rm -f /tmp/tmp.rs