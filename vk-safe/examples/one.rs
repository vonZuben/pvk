use vk_safe::safe_interface::{EnumerateInstanceExtensionProperties, EnumerateInstanceLayerProperties, EnumerateInstanceVersion};//, CreateInstance};

fn main() {
    let entry = vk_safe::entry::Entry::from_version(vk_safe_sys::VERSION_1_1).unwrap();

    println!("Supported Version: {}", entry.enumerate_instance_version().unwrap());

    // println!("num extensions available: {}", _x.enumerate_instance_extension_properties_len(None).unwrap());

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

//     let info: vk_safe_sys::InstanceCreateInfo = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
//     let instance = _x.create_instance(&info).unwrap();

//     println!("instance: {:?}", instance);

//     print!("SUCCESS!!!!");
}
