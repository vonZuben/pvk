use vk_safe::entry::*;
use vk_safe::safe_interface::structs::{ApplicationInfo, InstanceCreateInfo};

fn main() {
    let entry = vk_safe::entry::Entry::from_version(vk_safe_sys::VERSION_1_1).unwrap();

    println!("Supported Version: {}", entry.enumerate_instance_version().unwrap());

    println!("--Extensions--");
    for e in entry.enumerate_instance_extension_properties(None, Vec::new()).unwrap() {
        let s = e.extension_name();
        println!("{}", s);
    }

    println!("--Layers--");
    for e in entry.enumerate_instance_layer_properties(Vec::new()).unwrap() {
        println!("Name: {}", e.layer_name());
        println!("Description: {}", e.description());
        println!();
    }

    let app_info = ApplicationInfo::new(vk_safe_sys::VERSION_1_1);
    let instance_info = InstanceCreateInfo::new(&app_info);

    let instance = entry.create_instance(&instance_info).unwrap();

    println!("{instance:?}");

//     let info: vk_safe_sys::InstanceCreateInfo = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
//     let instance = _x.create_instance(&info).unwrap();

//     println!("instance: {:?}", instance);

//     print!("SUCCESS!!!!");
}
