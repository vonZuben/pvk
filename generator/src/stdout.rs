use generator::generate;

use quote::quote;

fn main() {
    let get_first_input_arg = || {
        let args = std::env::args_os();
        let current_exe = std::env::current_exe().unwrap();
        let current_exe = current_exe.to_string_lossy();
        for arg in args {
            let arg = arg.to_string_lossy();
            // check if the first arg is the program path
            // since this isn't technically guaranteed
            if current_exe.contains(arg.as_ref()) {
                continue;
            }
            else { // this should be the first real argument
                return arg.into_owned();
            }
        }
        panic!("no vk.xml path provided");
    };

    let code = generate(&get_first_input_arg());

    let code = append_main(code);

    println!("{}", code);
}

fn append_main(code: String) -> String {

    let main = quote!{
        fn main(){

            let mut instance = InstanceCreator::new()
                .enabled_extensions(&[KHR_get_physical_device_properties2])
                .app_name("heyo")
                .create()
                .unwrap();

            //let mut phd: PhysicalDevice = std::ptr::null();
            //let mut phd_count: u32 = 0;
            //instance_commands.EnumeratePhysicalDevices.0(inst, &mut phd_count as *mut u32, std::ptr::null_mut());
            let pd = instance.enumerate_physical_devices().unwrap();
            //println!("{:?}", inst);
            println!("{:?}", instance);
            for pd in &pd {
                println!("{:?}", pd);
                println!("{:?}", pd.get_physical_device_properties());
            }
            println!("num physical devices: {}", pd.len());

            // test Flags printing
            let flags: QueueFlags = QueueFlags::GRAPHICS | QueueFlags::COMPUTE;
            println!("{}", flags);
            println!("{:?}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::ALL;
            println!("{}", flags);
            println!("{:?}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::VERTEX
                | ShaderStageFlags::FRAGMENT | ShaderStageFlags::TESSELLATION_CONTROL;
            println!("{}", flags);
            println!("{:?}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::ALL_GRAPHICS;
            println!("{}", flags);
            println!("{:?}", flags);

            let u = ClearColorValue { int_32: [1, 1, 1, 1]};
            let i = ClearValue { color: u };
            println!("{:?}", i);


            // test DeviceCreator

            let queue_info = [
                DeviceQueueCreateInfo!{
                    queue_family_index: 0,
                    p_queue_priorities: &[1f32],
                }
            ];

            let pd1 = &pd[0];

            let device = unsafe { pd1.device_creator(&queue_info).create().unwrap() };
            println!("{:?}", device);

            let mut ma = MemoryAllocateInfo! {
                allocation_size: 1000u64,
                memory_type_index: 0u32,
            };

            let mut di = DedicatedAllocationMemoryAllocateInfoNV! {

            };

            ma.extend(&mut di);

            println!("{:?}", ma);

            let props = pd[0].get_physical_device_features_2::<(PhysicalDeviceVariablePointerFeatures, PhysicalDevice16BitStorageFeatures)>();
            println!("{:?}", props);
            println!("{:?}", props.pn().0);

            // #[derive(Copy, Clone)]
            // struct A<'owner>(PhantomData<&'owner ()>);
            // impl A<'_> {
            //     fn new() -> Self {
            //         Self(PhantomData)
            //     }
            // }
            // impl Handle for A<'_> {
            //     // type Owner = Own<'parent>;
            // }

            // struct Own<'parent> {
            //     handle: A<'static>,
            //     parent: &'parent InstanceOwner<'parent>,
            // }

            // impl<'parent> CreateOwner<'parent> for Own<'parent> {
            //     type Handle = A<'static>;
            //     type DispatchParent = InstanceOwner<'parent>;
            //     fn new(handle: Self::Handle, dispatch_parent: &'parent Self::DispatchParent) -> Self {
            //         Self {
            //             handle: handle,
            //             parent: dispatch_parent,
            //         }
            //     }
            // }

            // impl InstanceOwner<'_> {
            //     pub fn owner<'parent>(
            //         &'parent self,
            //     ) -> (Handles<'parent, Own<'parent>, Vec<A>>) {
            //         let v = vec![A::new(), A::new(), A::new(), A::new()];

            //         todo!()
            //         // ((v), self).ret()
            //     }
            // }

            // let o = instance.owner();

            // let mb = handle_slice![instance, instance];

            // for i in &mb {
            //     println!("{:?}", i);
            // }

            // let mb = mut_handle_slice![instance];

            // for i in &mb {
            //     println!("{:?}", i);
            // }

            //test 1_1 feature command ?
            //let mut phd: MaybeUninit<PhysicalDevice> = MaybeUninit::uninit();
            //let mut phd_count: u32 = 0;
            //instance_commands.EnumeratePhysicalDevices.0(inst, (&mut phd_count).into(), None.into());
            //println!("{}", phd_count);
        }
    };

    format!("{}{}", code, main)
}
