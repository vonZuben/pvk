# This is for testing the generated Vulkan code by generating and outputing it all into a single file

default:
	TMP_OUT_FILE=/tmp/tmp.rs cargo run --bin vk_stdout
	rustc /tmp/tmp.rs -o /tmp/tmp 2>/tmp/tmp.out
	/tmp/tmp
