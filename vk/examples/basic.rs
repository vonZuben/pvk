
/*
Notes of things to try and imporove

manually calling .handle() is kind of anoying

maybe some array arguments can be made as optional parameters such as in PipelineLayoutCreateInfo

ComputePipelineCreateInfo needs base_pipeline_index set which isn't needed unless a specifc flag is set

MyStr in public api is weird, should probably just take &CStr

*/


use std::{ffi::CString, io::Read};
use std::fs::File;

use vk::{self, ComputePipelineCreateInfo, DeviceQueueCreateInfo, MemoryPropertyFlags, PhysicalDeviceType, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo, QueueFlags, ShaderModuleCreateInfo, ShaderStageFlags};
use vk::HandleOwner;

fn main() {
    let instance = vk::InstanceCreator::new().create().unwrap();

    let physical_devices = instance.enumerate_physical_devices().unwrap();

    let get_integrated_pd = || {
        for pd in physical_devices {
            let props = pd.get_physical_device_properties();
            if props.device_type == PhysicalDeviceType::INTEGRATED_GPU {
                return Some(pd);
            }
        }
        None
    };

    let selected_pd = get_integrated_pd().unwrap();

    let memory_props = selected_pd.get_physical_device_memory_properties();

    let select_memory_type = || {
        for (i, mt) in memory_props.memory_types[..memory_props.memory_type_count as _].iter().enumerate() {
            if mt.property_flags.contains(MemoryPropertyFlags::DEVICE_LOCAL | MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT) {
                return Some(i);
            }
        }
        None
    };

    let selected_memory_type = select_memory_type().unwrap();

    let qp = selected_pd.get_physical_device_queue_family_properties();

    let get_compute_queue = || {
        for (i, p) in qp.iter().enumerate() {
            println!("{:?}", p);
            if p.queue_flags.contains(QueueFlags::COMPUTE) {
                return Some(i);
            }
        }
        None
    };

    let compute_queue_index = get_compute_queue().unwrap() as _;

    let device = unsafe {
        selected_pd.device_creator(&[DeviceQueueCreateInfo::new(compute_queue_index, &[1.0])])
            .create().unwrap()
    };


    // compute pipeline

    let mut buf = Vec::new();
    let code_size = File::open("shdr.spv").unwrap().read_to_end(&mut buf).unwrap();
    assert_eq!(code_size % 4, 0);
    let buf = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u32, code_size) };

    let shader_info = ShaderModuleCreateInfo::new(code_size, buf);
    let module = device.create_shader_module(&shader_info, None).unwrap();

    let c_string = CString::new("main").unwrap();
    let stage = PipelineShaderStageCreateInfo::new(ShaderStageFlags::COMPUTE, module.handle(), (&c_string).into());

    let pipe_layout_info = PipelineLayoutCreateInfo::new(&descriptor_layouts, &[]);
    let layout = device.create_pipeline_layout(&pipe_layout_info, None).unwrap();

    // WEIRD: the need to include base_pipeline_index here
    let cp_ci = ComputePipelineCreateInfo::new(stage, layout.handle(), 0);

    let cp = device.create_compute_pipelines(None, &[cp_ci], None).unwrap().pop().unwrap();


    println!("{:?}", device);
}