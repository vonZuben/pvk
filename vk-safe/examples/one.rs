use vk_safe::entry::EnumerateInstanceExtensionProperties;

trait AsStr {
    fn as_str(&self) -> &str;
}

impl<const N: usize> AsStr for [i8; N] {
    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(std::mem::transmute(&self[..self.len() - 1])) }
    }
}

fn main() {
    let _x = vk_safe::entry::Entry::<vk_safe_sys::version::entry::VERSION_1_0>::new().unwrap();

    println!("num extensions available: {}", _x.enumerate_instance_extension_properties_len(None).unwrap());

    for e  in _x.enumerate_instance_extension_properties(None).unwrap() {
        println!("{}", e.extension_name.as_str());
    }

    let mut props: [vk_safe_sys::ExtensionProperties; 3] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

    let res = _x.enumerate_instance_extension_properties_user(None, &mut props);

    println!("{:?}", res);

    println!("{:?}", props);

    print!("SUCCESS!!!!");
}