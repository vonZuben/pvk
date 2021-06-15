use vk::{self, HandleOwner};

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufWriter};

fn main() {
    unsafe { do_the_thing() }
}

unsafe fn do_the_thing() {
    let entry = vk::entry();
    let instance = entry.create_instance(vk::VERSION_1_0).unwrap();

    let selected_pd = instance
        .enumerate_physical_devices()
        .unwrap()
        .pop()
        .unwrap();

    println!("{:?}", selected_pd.get_physical_device_properties());

    let qp = selected_pd.get_physical_device_queue_family_properties();

    let get_render_queue = || {
        for (i, p) in qp.iter().enumerate() {
            if p.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                return Some(i);
            }
        }
        None
    };

    let render_queue_index = get_render_queue().unwrap() as _;

    let device_create_info = vk::DeviceQueueCreateInfo::new(render_queue_index, &[1.0]);
    let device = entry.device_params(&&selected_pd, &[device_create_info])
            .create_device(vk::VERSION_1_0)
            .unwrap();

    let vert_module = create_module(&device, "vk/examples/vert.spv");
    let frag_module = create_module(&device, "vk/examples/frag.spv");
    let entry = &std::ffi::CString::new("main").unwrap();
    let stages = [
        vk::PipelineShaderStageCreateInfo::new(
            vk::ShaderStageFlags::VERTEX,
            vert_module.handle(),
            entry.as_c_str().into(),
        ),
        vk::PipelineShaderStageCreateInfo::new(
            vk::ShaderStageFlags::FRAGMENT,
            frag_module.handle(),
            entry.as_c_str().into(),
        ),
    ];

    let raster_state = vk::PipelineRasterizationStateCreateInfo::new(
        0,
        0,
        vk::PolygonMode::FILL,
        vk::FrontFace::CLOCKWISE,
        0,
        0.0,
        0.0,
        0.0,
        1.0,
    ).cull_mode(vk::CullModeFlags::BACK);

    let layout_info = vk::PipelineLayoutCreateInfo::new();
    let layout = device.create_pipeline_layout(&layout_info, None).unwrap();

    let ca = [vk::AttachmentDescription::new(
        vk::Format::R8G8B8A8_UINT,
        vk::SampleCountFlags::TYPE_1,
        vk::AttachmentLoadOp::CLEAR,
        vk::AttachmentStoreOp::STORE,
        vk::AttachmentLoadOp::DONT_CARE,
        vk::AttachmentStoreOp::DONT_CARE,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    )];

    let cr = [vk::AttachmentReference::new(
        0,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    )];
    let sp =
        [vk::SubpassDescription::new(vk::PipelineBindPoint::GRAPHICS)
            .p_color_attachments(Some(&cr))];
    let rpinfo = vk::RenderPassCreateInfo::new(&sp).p_attachments(Some(&ca));
    let renderpass = device.create_render_pass(&rpinfo, None).unwrap();

    let mstate =
        vk::PipelineMultisampleStateCreateInfo::new(vk::SampleCountFlags::TYPE_1, 0, 1.0, 0, 0);
    let vp = [vk::Viewport::new(0.0, 0.0, 500.0, 500.0, 0.0, 1.0)];
    let scisors = [vk::Rect2D::new(
        vk::Offset2D::new(0, 0),
        vk::Extent2D::new(500, 500),
    )];
    let view_state = vk::PipelineViewportStateCreateInfo::new()
        .p_viewports(Some(&vp))
        .p_scissors(Some(&scisors));
    let input_state =
        vk::PipelineInputAssemblyStateCreateInfo::new(vk::PrimitiveTopology::TRIANGLE_LIST, 0);
    let batta = [vk::PipelineColorBlendAttachmentState::uninit()
        .blend_enable(0)
        .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
    ];
    let blend_state = vk::PipelineColorBlendStateCreateInfo::uninit().p_attachments(Some(&batta));
    let ivs = vk::PipelineVertexInputStateCreateInfo::new();
    let gpinfo = [vk::GraphicsPipelineCreateInfo::new(
        &stages,
        &raster_state,
        layout.handle(),
        renderpass.handle(),
        0,
        -1,
    )
    .p_multisample_state(Some(&mstate))
    .p_viewport_state(Some(&view_state))
    .p_input_assembly_state(Some(&input_state))
    .p_color_blend_state(Some(&blend_state))
    .p_vertex_input_state(Some(&ivs))];
    let render_pipeline = device
        .create_graphics_pipelines(None, &gpinfo, None)
        .unwrap()
        .pop()
        .unwrap();

    let imi = vk::ImageCreateInfo::new(
        vk::ImageType::TYPE_2D,
        vk::Format::R8G8B8A8_UINT,
        vk::Extent3D::new(500, 500, 1),
        1,
        1,
        vk::SampleCountFlags::TYPE_1,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::SharingMode::EXCLUSIVE,
        vk::ImageLayout::UNDEFINED,
    );
    let mut image = device.create_image(&imi, None).unwrap();

    let imr = device.get_image_memory_requirements(image.handle());

    let select_memory_type = || {
        let memory_props = selected_pd.get_physical_device_memory_properties();
        for (i, mt) in memory_props.memory_types[..memory_props.memory_type_count as _]
            .iter()
            .enumerate()
        {
            if mt.property_flags.contains(
                vk::MemoryPropertyFlags::DEVICE_LOCAL
                    | vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            ) {
                return Some(i);
            }
        }
        None
    };

    let mindex = select_memory_type().unwrap() as _;

    let mi = vk::MemoryAllocateInfo::new(imr.size, mindex);
    let memory = device.allocate_memory(&mi, None).unwrap();

    device.bind_image_memory(image.mut_handle(), memory.handle(), 0);

    let imvi = vk::ImageViewCreateInfo::new(
        image.handle(),
        vk::ImageViewType::TYPE_2D,
        vk::Format::R8G8B8A8_UINT,
        vk::ComponentMapping::new(
            vk::ComponentSwizzle::IDENTITY,
            vk::ComponentSwizzle::IDENTITY,
            vk::ComponentSwizzle::IDENTITY,
            vk::ComponentSwizzle::IDENTITY,
        ),
        vk::ImageSubresourceRange::new(vk::ImageAspectFlags::COLOR, 0, 1, 0, 1),
    );
    let imv = device.create_image_view(&imvi, None).unwrap();
    let imvs = [imv.handle()];
    let fbi =
        vk::FramebufferCreateInfo::new(renderpass.handle(), 500, 500, 1).p_attachments(Some(&imvs));
    let framebuffer = device.create_framebuffer(&fbi, None).unwrap();

    let cpi = vk::CommandPoolCreateInfo::new(render_queue_index);
    let mut cmd_pool = device.create_command_pool(&cpi, None).unwrap();

    let cai =
        vk::CommandBufferAllocateInfo::new(cmd_pool.handle(), vk::CommandBufferLevel::PRIMARY, 1);
    let mut cmdb = device
        .allocate_command_buffers(&cai)
        .unwrap()
        .pop()
        .unwrap();

    let bi = vk::CommandBufferBeginInfo::new();
    cmdb.begin_command_buffer(&bi);

    let clearvs = [vk::ClearValue {
        color: vk::ClearColorValue { int_32: [0; 4] },
    }];
    let rpi = vk::RenderPassBeginInfo::new(
        renderpass.handle(),
        framebuffer.handle(),
        vk::Rect2D::new(vk::Offset2D::new(0, 0), vk::Extent2D::new(500, 500)),
    )
    .p_clear_values(Some(&clearvs));
    cmdb.begin_render_pass(&rpi, vk::SubpassContents::INLINE);

    cmdb.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, render_pipeline.handle());
    cmdb.draw(3, 1, 0, 0);

    cmdb.end_render_pass();

    cmdb.end_command_buffer();

    // queue stuff
    let queue = device.get_device_queue(render_queue_index, 0);

    let buffers = [cmdb.handle()];
    let submits = [vk::SubmitInfo::new().p_command_buffers(Some(&buffers))];
    queue.queue_submit(&submits, None);

    queue.queue_wait_idle();

    let bi = vk::BufferCreateInfo::new(
        4 * 500 * 500,
        vk::BufferUsageFlags::TRANSFER_DST,
        vk::SharingMode::EXCLUSIVE,
    );
    let mut buffer = device.create_buffer(&bi, None).unwrap();

    let bmr = device.get_buffer_memory_requirements(buffer.handle());
    assert!((bmr.memory_type_bits & 1 << mindex) != 0);
    let mai = vk::MemoryAllocateInfo::new(bmr.size, mindex);
    let mut bmem = device.allocate_memory(&mai, None).unwrap();

    // let pbi = [vk::BindBufferMemoryInfo::new(buffer.handle(), bmem.handle(), 0)];
    device.bind_buffer_memory(buffer.mut_handle(), bmem.handle(), 0);

    device.reset_command_pool(cmd_pool.mut_handle(), None);

    let cai =
        vk::CommandBufferAllocateInfo::new(cmd_pool.handle(), vk::CommandBufferLevel::PRIMARY, 1);
    let mut cmdb = device
        .allocate_command_buffers(&cai)
        .unwrap()
        .pop()
        .unwrap();

    let bi = vk::CommandBufferBeginInfo::new();
    cmdb.begin_command_buffer(&bi);

    let regions = [vk::BufferImageCopy::new(
        0,
        500,
        500,
        vk::ImageSubresourceLayers::new(vk::ImageAspectFlags::COLOR, 0, 0, 1),
        vk::Offset3D::new(0, 0, 0),
        vk::Extent3D::new(500, 500, 1)
    )];
    cmdb.copy_image_to_buffer(
        image.handle(),
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        buffer.handle(),
        &regions,
    );

    cmdb.end_command_buffer();

    let buffers = [cmdb.handle()];
    let submits = [vk::SubmitInfo::new().p_command_buffers(Some(&buffers))];
    queue.queue_submit(&submits, None);

    queue.queue_wait_idle();

    let ptr = device
        .map_memory(bmem.mut_handle(), 0, vk::WHOLE_SIZE, None)
        .unwrap();
    let buf = std::slice::from_raw_parts(ptr as *const RGBApix, 500 * 500);
    // let buf = unsafe { std::slice::from_raw_parts(ptr as *const u8, 4 * 500 * 500) };
    // let buf2 = unsafe { std::slice::from_raw_parts(ptr as *const u32, 500 * 500) };
    // for f in buf.iter() {
    //     print!("{}, ", f);
    // }

    println!();

    #[repr(C)]
    struct RGBApix {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    }

    impl RGBApix {
        // fn r(&self) -> u8 { (self.0 >> 24).try_into().unwrap() }
        // fn g(&self) -> u8 { (self.0 >> 16 & 0xFF).try_into().unwrap() }
        // fn b(&self) -> u8 { (self.0 >> 8 & 0xFF).try_into().unwrap() }
        // fn r(&self) -> u8 { (self.0 & 0xFF).try_into().unwrap() }
        // fn g(&self) -> u8 { (self.0 >> 8 & 0xFF).try_into().unwrap() }
        // fn b(&self) -> u8 { (self.0 >> 16 & 0xFF).try_into().unwrap() }
        // fn
    }

    let out = OpenOptions::new()
        .create(true)
        .write(true)
        .open("out.ppm")
        .unwrap();

    out.set_len(0).unwrap();

    let mut out = BufWriter::new(out);

    let mut total = 0;
    let mut count = || {
        total += 1;
    };

    write!(out, "P3 500 500 255 ").unwrap();
    for (i, pix) in buf.iter().enumerate() {
        if i % 500 == 0 {
            write!(out, "\n").unwrap();
        }
        // println!("{:X}", pix);
        // println!("{:X}", buf2[i]);
        // let pix = Pix(*pix);
        // write!(out, "{} ", pix.r()).unwrap();
        // write!(out, "{} ", pix.g()).unwrap();
        // write!(out, "{} ", pix.b()).unwrap();
        write!(out, "{} ", pix.r).unwrap();
        write!(out, "{} ", pix.g).unwrap();
        write!(out, "{} ", pix.b).unwrap();
        write!(out, " ").unwrap();
        count();
    }

    assert!(dbg!(total) == 500 * 500);

    device.free_memory(memory, None);
    device.free_memory(bmem, None);
}

unsafe fn create_module<'device>(
    device: &'device vk::DeviceOwner,
    file: &str,
) -> vk::ShaderModuleOwner<'device, vk::Owned> {
    // compute pipeline
    const ASSUMED_BIG_ENOUGH: usize = 4096;
    let mut buf: Vec<u32> = Vec::with_capacity(ASSUMED_BIG_ENOUGH);
    let code_size = {
        println!("DIR {:?}", std::env::current_dir().unwrap());
        let buf = std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, ASSUMED_BIG_ENOUGH * 4);
        File::open(file).unwrap().read(buf).unwrap()
    };
    assert_eq!(code_size % 4, 0);
    assert!(code_size < ASSUMED_BIG_ENOUGH);

    buf.set_len(code_size / 4);
    // println!("code_sice: {} -> buf: {:?}", code_size, buf);

    let shader_info = vk::ShaderModuleCreateInfo::new(code_size, &buf);
    device.create_shader_module(&shader_info, None).unwrap()
}
