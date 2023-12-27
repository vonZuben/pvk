use vk_safe_sys::*;

instance_context!(MyCx: VERSION_1_0);

fn main() {
    use has_command::DestroyInstance;

    let mut instance = Instance {
        handle: std::ptr::null(),
    };
    let loader = |name| {
        // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
        unsafe { GetInstanceProcAddr(instance, name) }
    };

    let create_instance = CreateInstance::load(loader).unwrap();

    let mut info = unsafe { std::mem::MaybeUninit::<InstanceCreateInfo>::zeroed().assume_init() };
    info.s_type = StructureType::INSTANCE_CREATE_INFO;

    unsafe { create_instance.get_fptr()(&info, std::ptr::null(), &mut instance) };

    // reset since otherwise instance borrow is aliased
    let loader = |name| {
        // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
        unsafe { GetInstanceProcAddr(instance, name) }
    };

    let instance_commands = MyCx::load(loader).unwrap();

    println!("{:p}", instance_commands.DestroyInstance().get_fptr());
}
