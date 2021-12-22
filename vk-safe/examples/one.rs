use vk_safe::entry::EnermateExtensions;

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

    println!("num extensions available: {}", _x.num_extensions());

    for e  in _x.extensions() {
        println!("{}", e.extension_name.as_str());
    }

    print!("SUCCESS!!!");
}