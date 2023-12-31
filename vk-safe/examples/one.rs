// trace_macros!(true);
#![recursion_limit = "256"]

use vk_safe as vk;

use vk::vk_str;

vk::instance_context!(InstanceContext: VERSION_1_1);
vk::device_context!(DeviceContext: VERSION_1_0);

fn main() {
    println!(
        "Supported Version: {}",
        vk::enumerate_instance_version().unwrap()
    );

    println!("--Extensions--");
    for e in vk::enumerate_instance_extension_properties(None, Vec::new()).unwrap() {
        println!("{e:#?}");
    }

    println!("--Layers--");
    for e in vk::enumerate_instance_layer_properties(Vec::new()).unwrap() {
        println!("{e:#?}");
    }

    const INSTANCE_INFO: vk::InstanceCreateInfo<InstanceContext::InstanceContext> =
        vk::InstanceCreateInfo::new(
            &vk::ApplicationInfo::new(InstanceContext)
                .app_name(vk_str!("Example"))
                .app_version(vk::VkVersion::new(0, 0, 1)),
        );

    let instance = vk::create_instance(&INSTANCE_INFO).unwrap();

    println!("-------");
    println!("{instance:?}");

    vk::scope(&instance, |instance| {
        let physical_devices = instance
            .enumerate_physical_devices([std::mem::MaybeUninit::uninit(); 1])
            .unwrap();

        println!("-------");
        println!("{physical_devices:?}");

        println!("-------");
        for pd in physical_devices.iter() {
            vk::scope(&pd, |pd| {
                println!("{:#?}", pd.get_physical_device_properties());
                println!("-------");

                println!("{:#?}", pd.get_physical_device_features());

                println!("---Device Extensions----");
                println!(
                    "{:#?}",
                    pd.enumerate_device_extension_properties(None, Vec::new())
                        .unwrap()
                );

                println!("---Device Layers----");
                println!(
                    "{:#?}",
                    pd.enumerate_device_layer_properties(Vec::new()).unwrap()
                );

                //test getting format properties
                let srgb_properties =
                    pd.get_physical_device_format_properties(vk::format::R8G8B8A8_SRGB);
                println!("R8G8B8A8_SRGB: {srgb_properties:#?}");

                const PARAMS: vk::GetPhysicalDeviceImageFormatPropertiesParameters =
                    vk::GetPhysicalDeviceImageFormatPropertiesParameters::new(
                        vk::Format::R8G8B8A8_SRGB,
                        vk::ImageType::TYPE_2D,
                        vk::ImageTiling::OPTIMAL,
                        vk::ImageUsageFlags::COLOR_ATTACHMENT_BIT
                            .or(vk::ImageUsageFlags::TRANSFER_DST_BIT),
                        vk::ImageCreateFlags::empty(),
                    );
                //test image format properties
                let tst_image_format_properties = pd
                    .get_physical_device_image_format_properties(PARAMS)
                    .unwrap();
                println!("{tst_image_format_properties:#?}");
                let sparse_image_format_properties = pd
                    .get_physical_device_sparse_image_format_properties(
                        vk::SampleCountFlags::TYPE_1_BIT,
                        tst_image_format_properties,
                        Vec::new(),
                    )
                    .unwrap();
                println!("---spare properties for above image format properties----");
                println!("{sparse_image_format_properties:#?}");

                println!("-------");
                let queue_family_properties = pd
                    .get_physical_device_queue_family_properties(Vec::new())
                    .unwrap();
                println!("{:#?}", queue_family_properties);
                println!("-------");
                let mem_props = pd.get_physical_device_memory_properties();
                println!("{:#?}", mem_props);

                // just assume that 10 is the max number of queues we will see fo now
                let standard_queue_priorities =
                    unsafe { vk::QueuePriorities::new_unchecked(&[1.0; 10]) };

                let queue_family_configurations =
                    queue_family_properties.configure_create_info(Vec::new(), |config| {
                        let queue_count = config.family_properties.queue_count;
                        config
                            .push_config(
                                standard_queue_priorities.with_num_queues(queue_count as _),
                                vk::DeviceQueueCreateFlags::empty(),
                            )
                            .expect("problem writing queue config");
                    });

                println!("{:#?}", queue_family_configurations);

                let device_create_info =
                    vk::DeviceCreateInfo::new(DeviceContext, &queue_family_configurations);

                let device = pd.create_device(&device_create_info).unwrap();

                vk::scope(&device, |device| {
                    let mem_type = mem_props.choose_type(0);
                    let alloc_info = vk::MemoryAllocateInfo::new(
                        std::num::NonZeroU64::new(100).unwrap(),
                        mem_type,
                    );
                    let mem = device.allocate_memory(&alloc_info);
                    println!("{mem:?}");
                })();

                println!("-------");
                println!("{device:#?}");
            })();
        }
    })();
}
