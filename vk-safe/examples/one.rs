// trace_macros!(true);

/*!
This is an example program that grows along side with the development of the vk-safe API in order to test all functions together.
Thus, I try to make use of all APIs here as I add them.
 */

use vk_safe::vk;
use vk_safe::vk_str;

use vk::traits::*;

vk::instance_context!(InstanceContext: VERSION_1_1 + KHR_surface);
vk::device_context!(DeviceContext: VERSION_1_0);

fn main() {
    println!(
        "Supported Version: {}",
        vk::enumerate_instance_version().unwrap()
    );

    println!("--Supported instance extensions--");
    for e in vk::enumerate_instance_extension_properties(None, Vec::new()).unwrap() {
        println!("{e:#?}");
    }

    println!("--Available instance layers--");
    for e in vk::enumerate_instance_layer_properties(Vec::new()).unwrap() {
        println!("{e:#?}");
    }

    vk::tag!(instance_tag);

    let app_info = vk::ApplicationInfo::new(InstanceContext)
        .app_name(vk_str!("Example App"))
        .app_version(vk_safe::VkVersion::new(0, 0, 1));
    let instance_info = vk::InstanceCreateInfo::new(&app_info);
    let instance = vk::create_instance(&instance_info, instance_tag).unwrap();

    println!("--Example Instance handle--");
    println!("{instance:?}");

    let physical_devices = instance
        .enumerate_physical_devices([std::mem::MaybeUninit::uninit(); 1])
        .unwrap();

    println!("--Physical Devices on the system--");
    println!("{physical_devices:?}");

    println!("--For Each Physical Device--");
    for pd in physical_devices.iter() {
        std::thread::scope(|scope| {
            scope.spawn(|| {
                vk::tag!(tag);
                run_physical_device(pd.tag(tag))
            });
        });
    }
}

fn run_physical_device(pd: impl PhysicalDevice<Commands: vk::instance::VERSION_1_0>) {
    println!("-------");
    println!("{:#?}", pd.get_physical_device_properties());

    println!("-------");
    println!("{:#?}", pd.get_physical_device_features());

    println!("--Supported device extensions--");
    println!(
        "{:#?}",
        pd.enumerate_device_extension_properties(None, Vec::new())
            .unwrap()
    );

    println!("--Available device layers (NOTE: device layers are depreciated by Vulkan)--");
    println!(
        "{:#?}",
        pd.enumerate_device_layer_properties(Vec::new()).unwrap()
    );

    let srgb_properties = pd.get_physical_device_format_properties(vk::format::R8G8B8A8_SRGB);
    println!("--Example of device format propertied for R8G8B8A8_SRGB:");
    println!("{srgb_properties:#?}");

    const PARAMS: vk::ImageParameters = vk::ImageParameters::new(
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageType::TYPE_2D,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT_BIT.or(vk::ImageUsageFlags::TRANSFER_DST_BIT),
        vk::ImageCreateFlags::empty(),
    );

    let tst_image_format_properties = pd
        .get_physical_device_image_format_properties(PARAMS)
        .unwrap();
    println!("--Example of device format propertied for R8G8B8A8_SRGB, 2D, Optimal tiling, to be used as Transfer destination and Color attachment:");
    println!("{tst_image_format_properties:#?}");

    let sparse_image_format_properties = pd
        .get_physical_device_sparse_image_format_properties(
            vk::SampleCountFlags::TYPE_1_BIT,
            tst_image_format_properties,
            Vec::new(),
        )
        .unwrap();
    println!("--Example sparse properties for above image format properties--");
    println!("{sparse_image_format_properties:#?}");

    let queue_family_properties = pd
        .get_physical_device_queue_family_properties(Vec::new())
        .unwrap();
    println!("--Queue family properties for this physical device--");
    println!("{:#?}", queue_family_properties);

    let mem_props = pd.get_physical_device_memory_properties();
    println!("--Memory properties for this physical device--");
    println!("{:#?}", mem_props);

    vk::tag!(families_tag);

    let mut queue_configs = vec![];
    let priorities = [vk::QueuePriority::default(); 10];
    for p in queue_family_properties.properties_iter(families_tag) {
        use vk::queue_flag_bits::*;
        if p.queue_flags
            .contains(GRAPHICS_BIT | COMPUTE_BIT | TRANSFER_BIT)
        {
            queue_configs.push(
                vk::DeviceQueueCreateInfo::new(&priorities[..p.queue_count as usize], p).unwrap(),
            )
        }
    }

    let device_create_info = vk::DeviceCreateInfo::new(DeviceContext, &queue_configs);

    vk::tag!(dt);
    let device = pd.create_device(&device_create_info, dt).unwrap();

    println!("--Example Device handle--");
    println!("{device:#?}");

    vk::flags!(MemProps: MemoryPropertyFlags + HOST_VISIBLE_BIT);
    vk::flags!(HeapBits: MemoryHeapFlags - MULTI_INSTANCE_BIT);

    let mem_type = mem_props.find_ty(MemProps, HeapBits).unwrap();
    let alloc_info = vk::MemoryAllocateInfo::new(std::num::NonZeroU64::new(100).unwrap(), mem_type);
    let mem = device.allocate_memory(&alloc_info).unwrap();
    println!("--Example allocated memory handle--");
    println!("{mem:?}");

    let mapped_memory = device.map_memory(mem).unwrap();
    println!("--Example mapped memory handle--");
    println!("{mapped_memory:#?}");

    let ranges = [vk::MappedMemoryRange::whole_range(&mapped_memory)];
    device.flush_mapped_memory_ranges(&ranges).unwrap();

    let _memory = device.unmap_memory(mapped_memory);

    // vk::flags!(QCaps: QueueFlags + GRAPHICS_BIT + TRANSFER_BIT + COMPUTE_BIT);
    // for queue_config in queue_configs {
    //     let queue_family = device
    //         .get_queue_family(&queue_config, &queue_family_properties, QCaps)
    //         .unwrap();
    //     println!("Configured Queue Family: {:#?}", queue_family);

    //     let queue = queue_family.get_device_queue(0).unwrap();
    //     println!("Queue: {:#?}", queue);
    // }

    unsafe {
        // safe since everything is one one thread
        device.wait_idle().unwrap();
    }
}
