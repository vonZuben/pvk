use generator::parse_vk_xml;

fn main() {
    println!("{:?}", std::env::current_dir());
    let code = parse_vk_xml("generator/vk.xml");

    let util_code = code.versions();
    println!("{util_code}");
}
