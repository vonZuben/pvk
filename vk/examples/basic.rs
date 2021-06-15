/*
Notes of things to try and imporove

manually calling .handle() is kind of anoying

maybe some array arguments can be made as optional parameters such as in PipelineLayoutCreateInfo

ComputePipelineCreateInfo needs base_pipeline_index set which isn't needed unless a specifc flag is set

MyStr in public api is weird, should probably just take &CStr

ShaderModuleCreateInfo::new asks for code_size (shoudn't)

providing some kind of shader module loading helper would be nice (awkward going between raw u8 buffer to u32 buffer manually)

NOT freeing "DeviceMemory" is actually a validation layer, so not automatically dropping it is actually a problem

need better handling of Array<c_void>

SpecializationInfo is an example where the map and data members need to be carfully set toghether
also, if one of the members is not provided, there is no validation trigger, and very undefined behaviour is caused

*/

use std::fs::File;
use std::{ffi::CString, io::Read};

use vk::ex;

use vk::HandleOwner;
use vk::{
    self, BufferCreateInfo, BufferUsageFlags, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPoolCreateInfo, ComputePipelineCreateInfo, DescriptorBufferInfo,
    DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSetAllocateInfo,
    DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
    DeviceQueueCreateInfo, MemoryAllocateInfo, MemoryPropertyFlags, PhysicalDeviceType,
    PipelineBindPoint, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo, QueueFlags,
    ShaderModuleCreateInfo, ShaderStageFlags, SharingMode, WriteDescriptorSet,
};

fn main() {
    unsafe { do_the_thing() }
}

unsafe fn do_the_thing() {
    let version = vk::enumerate_instance_version().unwrap();
    println!("{:?}", vk::VkVersion::from_raw(version));
    let entry = vk::entry() //.api_version(vk::VERSION_1_1)
        .enabled_instance_extensions(ex![vk::KHR_surface, vk::KHR_swapchain]);
    let instance = entry.create_instance(vk::VERSION_1_1).unwrap();

    let physical_devices = instance.enumerate_physical_devices().unwrap();

    let get_integrated_pd = || {
        for pd in physical_devices.iter() {
            let props = pd.get_physical_device_properties();
            println!("{:#?}", props);
            if props.device_type == PhysicalDeviceType::INTEGRATED_GPU {
                return Ok(pd);
            }
        }
        Err("Can't find requested type of device")
    };

    let selected_pd = get_integrated_pd().unwrap();

    // panic!();

    let memory_props = selected_pd.get_physical_device_memory_properties();

    let select_memory_type = || {
        for (i, mt) in memory_props.memory_types[..memory_props.memory_type_count as _]
            .iter()
            .enumerate()
        {
            if mt.property_flags.contains(
                MemoryPropertyFlags::DEVICE_LOCAL
                    | MemoryPropertyFlags::HOST_VISIBLE
                    | MemoryPropertyFlags::HOST_COHERENT,
            ) {
                return Some(i);
            }
        }
        None
    };

    let selected_memory_type = select_memory_type().unwrap();

    let qp = selected_pd.get_physical_device_queue_family_properties();

    let get_compute_queue = || {
        for (i, p) in qp.iter().enumerate() {
            // println!("{:?}", p);
            if p.queue_flags.contains(QueueFlags::COMPUTE) {
                return Some(i);
            }
        }
        None
    };

    let compute_queue_index = get_compute_queue().unwrap() as _;

    let device_create_info = DeviceQueueCreateInfo::new(compute_queue_index, &[1.0]);
    let device = entry.make_device(&selected_pd, &[device_create_info])
            .create_device(vk::VERSION_1_0)
            .unwrap();

    let queue = device.get_device_queue(compute_queue_index, 0);

    // compute pipeline
    const ASSUMED_BIG_ENOUGH: usize = 4096;
    let mut buf: Vec<u32> = Vec::with_capacity(ASSUMED_BIG_ENOUGH);
    let code_size = {
        println!("DIR {:?}", std::env::current_dir().unwrap());
        let buf = std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, ASSUMED_BIG_ENOUGH * 4);
        File::open("vk/examples/comp.spv")
            .unwrap()
            .read(buf)
            .unwrap()
    };
    assert_eq!(code_size % 4, 0);
    assert!(code_size < ASSUMED_BIG_ENOUGH);

    buf.set_len(code_size / 4);
    // println!("code_sice: {} -> buf: {:?}", code_size, buf);

    let shader_info = ShaderModuleCreateInfo::new(code_size, &buf);
    let module = device.create_shader_module(&shader_info, None).unwrap();
    eprintln!("shrd_module {:?}", module);

    let number_of_elements = selected_pd
        .get_physical_device_properties()
        .limits
        .max_compute_work_group_size[0];

    let map = [vk::SpecializationMapEntry::new(
        0,
        0,
        std::mem::size_of::<u32>(),
    )];
    let special = vk::SpecializationInfo::new()
        .p_map_entries(Some(&map))
        .p_data(Some(vk::OpaqueData::from(&number_of_elements)));
    println!("{:#?}", special);
    let c_string = CString::new("main").unwrap();
    let stage = PipelineShaderStageCreateInfo::new(
        ShaderStageFlags::COMPUTE,
        module.handle(),
        c_string.as_c_str().into(),
    )
    .p_specialization_info(Some(&special));

    let b1 = DescriptorSetLayoutBinding::new(
        0,
        DescriptorType::STORAGE_BUFFER,
        ShaderStageFlags::COMPUTE,
    )
    .descriptor_count(1);
    // println!("{:#?}", b1);
    // println!("{:#?}", DescriptorSetLayoutBinding::uninit());
    let bindings = [b1];
    let dsl_info = DescriptorSetLayoutCreateInfo::new().p_bindings(Some(&bindings));
    let ds_layout = device
        .create_descriptor_set_layout(&dsl_info, None)
        .unwrap();
    eprintln!("ds_layout {:?}", ds_layout);
    let descriptor_layouts = [ds_layout.handle()];
    let pipe_layout_info = PipelineLayoutCreateInfo::new().p_set_layouts(Some(&descriptor_layouts));
    let layout = device
        .create_pipeline_layout(&pipe_layout_info, None)
        .unwrap();
    eprintln!("layout {:?}", layout);

    // WEIRD: the need to include base_pipeline_index here
    let cp_ci = ComputePipelineCreateInfo::new(stage, layout.handle(), -1);

    let cpipeline = device
        .create_compute_pipelines(None, &[cp_ci], None)
        .unwrap()
        .pop()
        .unwrap();
    eprintln!("cpipeline {:?}", cpipeline);

    // buffer

    let buff_info = BufferCreateInfo::new(
        std::mem::size_of::<f32>() as u64 * number_of_elements as u64,
        BufferUsageFlags::STORAGE_BUFFER,
        SharingMode::EXCLUSIVE,
    );
    let mut buffer = device.create_buffer(&buff_info, None).unwrap();
    eprintln!("buffer {:?}", buffer);

    let mem_req = device.get_buffer_memory_requirements(buffer.handle());
    assert!((mem_req.memory_type_bits & 1 << selected_memory_type) != 0);
    let mem_info = MemoryAllocateInfo::new(mem_req.size, selected_memory_type as _);
    let mut memory = device.allocate_memory(&mem_info, None).unwrap();
    eprintln!("memory {:?}", memory);

    device.bind_buffer_memory(buffer.mut_handle(), memory.handle(), 0);

    // descriptor set

    let pool_size = DescriptorPoolSize::new(DescriptorType::STORAGE_BUFFER, 1);
    let pool_sizes = &[pool_size];
    let ds_pool_info = DescriptorPoolCreateInfo::new(1, pool_sizes);
    let ds_pool = device.create_descriptor_pool(&ds_pool_info, None).unwrap();
    eprintln!("ds_pool {:?}", ds_pool);

    let layouts = &[ds_layout.handle()];
    let ds_alloc_info = DescriptorSetAllocateInfo::new(ds_pool.handle(), layouts);
    let ds = device
        .allocate_descriptor_sets(&ds_alloc_info)
        .unwrap()
        .pop()
        .unwrap();

    let buffer_write_info = DescriptorBufferInfo::new(buffer.handle(), 0, vk::WHOLE_SIZE);
    let buffer_writes = [buffer_write_info];
    let write = WriteDescriptorSet::new(ds.handle(), 0, 0, DescriptorType::STORAGE_BUFFER)
        .p_buffer_info(Some(&buffer_writes));
    device.update_descriptor_sets(&[write], &[]);

    // command buffer

    let cmd_pool_info = CommandPoolCreateInfo::new(compute_queue_index);
    let cmd_pool = device.create_command_pool(&cmd_pool_info, None).unwrap();
    eprintln!("cmd_pool {:?}", cmd_pool);

    let cmd_alloc_info =
        CommandBufferAllocateInfo::new(cmd_pool.handle(), CommandBufferLevel::PRIMARY, 1);
    let mut cmd_buffer = device
        .allocate_command_buffers(&cmd_alloc_info)
        .unwrap()
        .pop()
        .unwrap();

    let begin_info = CommandBufferBeginInfo::new(); //.flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    cmd_buffer.begin_command_buffer(&begin_info);

    cmd_buffer.bind_pipeline(PipelineBindPoint::COMPUTE, cpipeline.handle());
    cmd_buffer.bind_descriptor_sets(
        PipelineBindPoint::COMPUTE,
        layout.handle(),
        0,
        &[ds.handle()],
        &[],
    );

    cmd_buffer.dispatch(1, 1, 1);

    cmd_buffer.end_command_buffer();

    let cmd_buffers = &[cmd_buffer.handle()];
    let submit = vk::SubmitInfo::new().p_command_buffers(Some(cmd_buffers));
    let submits = [submit];
    queue.queue_submit(&submits, None);

    queue.queue_wait_idle();

    let ptr = device
        .map_memory(memory.mut_handle(), 0, vk::WHOLE_SIZE, None)
        .unwrap();
    let buf = std::slice::from_raw_parts(ptr as *const f32, number_of_elements as _);
    println!("there were {} invocations", number_of_elements);
    for f in buf.iter() {
        print!("{}, ", f);
    }

    // device.unmap_memory(memory.mut_handle());

    // I should not need to call this manually
    device.free_memory(memory, None);
}
