// use vk_safe::safe_interface::{EnumerateInstanceExtensionProperties, EnumerateInstanceLayerProperties, EnumerateInstanceVersion, CreateInstance};

fn main() {
//     let _x = vk_safe::entry::Entry::from_version(vk_safe_sys::version::VERSION_1_1).unwrap();

//     println!("num extensions available: {}", _x.enumerate_instance_extension_properties_len(None).unwrap());

//     for e in _x.enumerate_instance_extension_properties(None, Vec::new()).unwrap() {
//         let s = e.extension_name();
//         println!("{}", s);
//     }

//     for e in _x.enumerate_instance_layer_properties(Vec::new()).unwrap() {
//         println!("{}", e.layer_name());
//         println!("{}", e.description());
//     }

//     println!("Version: {}", _x.enumerate_instance_version().unwrap());

//     let info: vk_safe_sys::InstanceCreateInfo = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
//     let instance = _x.create_instance(&info).unwrap();

//     println!("instance: {:?}", instance);

//     print!("SUCCESS!!!!");
}
