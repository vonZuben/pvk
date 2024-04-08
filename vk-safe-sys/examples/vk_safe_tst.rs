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

    struct Commands<C: context::Commands> {
        c: C::Commands,
    }

    impl<C: context::Commands> Commands<C> {
        fn load(instance: Instance, _: C) -> Self {
            // reset since otherwise instance borrow is aliased
            let loader = |name| {
                // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
                unsafe { GetInstanceProcAddr(instance, name) }
            };
            Self {
                c: C::Commands::load(loader).unwrap(),
            }
        }
    }

    let instance_commands = Commands::load(instance, MyCx);

    println!("{:p}", instance_commands.c.DestroyInstance().get_fptr());
}
