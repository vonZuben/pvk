use vk_safe::safe_interface::{EnumerateInstanceExtensionProperties, EnumerateInstanceLayerProperties, CreateInstance};

trait AsStr {
    fn as_str(&self) -> &str;
}

impl<const N: usize> AsStr for [i8; N] {
    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(std::mem::transmute(&self[..self.len() - 1])) }
    }
}

fn main() {
    let _x = vk_safe::entry::Entry::from_version(vk_safe_sys::version::VERSION_1_0).unwrap();

    println!("num extensions available: {}", _x.enumerate_instance_extension_properties_len(None).unwrap());

    for e in _x.enumerate_instance_extension_properties(None, Vec::new()).unwrap() {
        println!("{}", e.extension_name.as_str());
    }

    for e in _x.enumerate_instance_layer_properties(Vec::new()).unwrap() {
        println!("{}", e.layer_name.as_str());
    }

    let info: vk_safe_sys::InstanceCreateInfo = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    let instance = _x.create_instance(&info).unwrap();

    println!("instance: {:?}", instance);

    print!("SUCCESS!!!!");
}
