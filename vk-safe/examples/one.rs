// trace_macros!(true);
#![recursion_limit = "256"]

use vk_safe::entry::*;
use vk_safe::instance::Config;
// use vk_safe::instance::*;
use vk_safe::scope::scope;

use vk_safe_sys as vk;
use vk_safe::vk_str;

fn main() {
    let entry = vk_safe::entry::Entry::from_version(vk_safe_sys::VERSION_1_1).unwrap();

    println!(
        "Supported Version: {}",
        entry.enumerate_instance_version().unwrap()
    );

    println!("--Extensions--");
    for e in entry
        .enumerate_instance_extension_properties(None, Vec::new())
        .unwrap()
    {
        println!("{e:#?}");
    }

    println!("--Layers--");
    for e in entry
        .enumerate_instance_layer_properties(Vec::new())
        .unwrap()
    {
        println!("{e:#?}");
    }

    let instance_config = Config::new(vk_safe_sys::VERSION_1_1, ());
    let app_info = ApplicationInfo::new(instance_config)
        .app_name(vk_str!("My App"))
        .app_version(vk_safe::VkVersion::new(0, 0, 1));
    let instance_info = InstanceCreateInfo::new(&app_info);

    let instance = entry.create_instance(&instance_info).unwrap();

    println!("-------");
    println!("{instance:?}");

    scope(instance, |instance| {
        let physical_devices = instance
        .enumerate_physical_devices([std::mem::MaybeUninit::uninit(); 1])
        .unwrap();

        println!("-------");
        println!("{physical_devices:?}");

        println!("-------");
        for pd in physical_devices.iter() {
            scope(pd, |pd| {
                println!("{:#?}", pd.get_physical_device_properties());
                println!("-------");

                println!("{:#?}", pd.get_physical_device_features());

                //test getting format properties
                let srgb_properties = pd.get_physical_device_format_properties(vk::format::R8G8B8A8_SRGB);
                println!("R8G8B8A8_SRGB: {srgb_properties:#?}");

                const PARAMS: vk_safe::physical_device::GetPhysicalDeviceImageFormatPropertiesParams = vk_safe::physical_device::GetPhysicalDeviceImageFormatPropertiesParams::new(
                    vk::Format::R8G8B8A8_SRGB,
                    vk::ImageType::TYPE_2D,
                    vk::ImageTiling::OPTIMAL,
                    vk::ImageUsageFlags::COLOR_ATTACHMENT_BIT.or(vk::ImageUsageFlags::TRANSFER_DST_BIT),
                    vk::ImageCreateFlags::empty()
                );
                //test image format properties
                let tst_image_format_properties = pd.get_physical_device_image_format_properties(PARAMS).unwrap();
                println!("{tst_image_format_properties:#?}");
                println!("-------");
                let queue_family_properties = pd.get_physical_device_queue_family_properties(Vec::new());
                println!("{:#?}", queue_family_properties);
                println!("-------");
                let mem_props = pd.get_physical_device_memory_properties();
                println!("{:#?}", mem_props);

                // just assume that 10 is the max number of queues we will see fo now
                let standard_queue_priorities = unsafe { vk_safe::physical_device::QueuePriorities::new_unchecked(&[1.0;10]) };

                let queue_family_configurations = queue_family_properties.configure_create_info(Vec::new(), |config| {
                    let queue_count = config.family_properties.queue_count;
                    config.push_config(standard_queue_priorities.with_num_queues(queue_count as _), vk::DeviceQueueCreateFlags::empty()).expect("problem writing queue config");
                });

                println!("{:#?}", queue_family_configurations);

                let device_config = vk_safe::device::Config::new(vk_safe_sys::VERSION_1_1, ());

                let device_create_info = vk_safe::physical_device::DeviceCreateInfo::new(device_config, &queue_family_configurations);

                let device = pd.create_device(&device_create_info).unwrap();

                scope(&device, |device: vk_safe::scope::Scope<'_, vk_safe::device::Device<vk_safe::device::Config<vk::VERSION_1_1, ()>, vk_safe::scope::Scope<'_, vk_safe::physical_device::PhysicalDevice<'_, Config<vk::VERSION_1_1, ()>>>>>| {
                    let mem_type = mem_props.choose_type(0);
                    let alloc_info = vk_safe::device::allocate_memory::MemoryAllocateInfo::new(std::num::NonZeroU64::new(100).unwrap(), mem_type);
                    let mem = device.allocate_memory(&alloc_info);
                    println!("{mem:?}");
                })();

                println!("-------");
                println!("{device:#?}");
            })();
        }
    })();
}
