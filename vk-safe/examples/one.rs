// trace_macros!(true);
#![recursion_limit = "256"]

use vk_safe as vk;

use vk::vk_str;

vk::instance_context!(InstanceContext: VERSION_1_1 + KHR_wayland_surface + KHR_surface);
vk::device_context!(DeviceContext: VERSION_1_0 + KHR_external_fence_fd + KHR_external_fence);

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

    let app_info = vk::ApplicationInfo::new(InstanceContext)
        .app_name(vk_str!("Example"))
        .app_version(vk::VkVersion::new(0, 0, 1));

    let instance_info = vk::InstanceCreateInfo::new(&app_info);

    let instance = vk::create_instance(&instance_info).unwrap();

    println!("-------");
    println!("{instance:?}");

    vk::scope(instance, |instance| {
        let physical_devices = instance
            .enumerate_physical_devices([std::mem::MaybeUninit::uninit(); 1])
            .unwrap();

        println!("-------");
        println!("{physical_devices:?}");

        println!("-------");
        for pd in physical_devices.iter() {
            std::thread::scope(|scope| {
                scope.spawn(vk::scope(pd, |pd| run_physical_device(pd)));
            });
        }
    })();
}

fn run_physical_device<C: vk::instance::VERSION_1_0>(pd: impl vk::PhysicalDevice<Commands = C>) {
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
    let srgb_properties = pd.get_physical_device_format_properties(vk::format::R8G8B8A8_SRGB);
    println!("R8G8B8A8_SRGB: {srgb_properties:#?}");

    const PARAMS: vk::GetPhysicalDeviceImageFormatPropertiesParameters =
        vk::GetPhysicalDeviceImageFormatPropertiesParameters::new(
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageType::TYPE_2D,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT_BIT.or(vk::ImageUsageFlags::TRANSFER_DST_BIT),
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

    queue_family_properties.config_scope(|qp| {
        let mut queue_configs = vec![];
        let priorities = vk::QueuePriorities::new(&[1.0; 10]);
        for p in qp {
            if p.queue_flags.contains(vk::QueueFlags::GRAPHICS_BIT) {
                queue_configs.push(
                    vk::DeviceQueueCreateInfo::new(priorities.with_num_queues(p.queue_count), p)
                        .unwrap(),
                )
            }
        }

        let device_create_info = vk::DeviceCreateInfo::new(DeviceContext, &queue_configs);

        let device = pd
            .create_device(&device_create_info, &queue_family_properties)
            .unwrap();

        vk::scope(device, |device| {
            let mem_type = mem_props.choose_type(0);
            let alloc_info =
                vk::MemoryAllocateInfo::new(std::num::NonZeroU64::new(100).unwrap(), mem_type);
            let mem = device.allocate_memory(&alloc_info);
            println!("{mem:?}");

            println!("-------");
            println!("{device:#?}");

            for qf in device.get_configured_queue_families() {
                println!("queue family: {:#?}", qf);

                vk::queue_capabilities!(QCaps: GRAPHICS_BIT, COMPUTE_BIT, TRANSFER_BIT);
                let qf = qf.with_capability(QCaps).unwrap();

                let queue = qf.get_queue(0);
                println!("{queue:#?}");
            }
        })();
    });
}
