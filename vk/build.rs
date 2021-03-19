
use std::{io::{Read, Write}, process::{Command, Stdio}};
use std::fs::OpenOptions;

use generator::generate;

fn main() {
    let fmt = Command::new("rustfmt")
        .args(&["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Error: cannot run rustfmt");

    fmt.stdin.expect("Error: can't get rustfmt stdin").write(generate("vk.xml").as_bytes()).expect("Error writting to rustfmt stdin");

    let mut formatted_code = Vec::new();
    fmt.stdout.expect("Error: faild to get formatted code").read_to_end(&mut formatted_code).expect("can't read from formatted code stdout");

    let mut vkrs = OpenOptions::new()
        .write(true)
        .create(true)
        .open("src/vk.rs")
        .expect("Error: cannot open vk.rs for writting");

    vkrs.write(&formatted_code).expect("Error: could not write to vk.rs");
    vkrs.set_len(formatted_code.len() as _).expect("Error: cannot set vk.rs file len");
}
