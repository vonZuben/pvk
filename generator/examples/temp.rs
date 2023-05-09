use generator::parse_vk_xml;

fn main() {
    println!("{:?}", std::env::current_dir());
    let code = parse_vk_xml("generator/vk.xml", "generator/validusage.json");

    let util_code = code.aliases();
    println!("{util_code}");
}
