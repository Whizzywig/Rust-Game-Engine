extern crate cgmath;
extern crate winit;
extern crate time;

#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate tobj;

use vulkano_win::VkSurfaceBuild;
use vulkano::sync::GpuFuture;

use std::thread;
use std::sync::Arc;

mod support;
mod math;
mod init;

fn main() {
    let extensions = vulkano_win::required_extensions();
    let instance = vulkano::instance::Instance::new(None, &extensions, None).expect("failed to create instance");

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance).next().expect("No device available");
    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    let mut events_loop = winit::EventsLoop::new();
    //sets up the window
    let surface = winit::WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
    surface.window().grab_cursor(true).expect("Failed to grab cursor");
    surface.window().hide_cursor(true);


    let mut dimensions;

    let queue_family = physical.queue_families().find(|&q| q.supports_graphics() &&
        surface.is_supported(q).unwrap_or(false)).expect("Couldn't find a graphical queue family");

    let device_ext = vulkano::device::DeviceExtensions {
        khr_swapchain: true,
        .. vulkano::device::DeviceExtensions::none()
    };

    let (device, mut queues) = vulkano::device::Device::new(physical, physical.supported_features(),
                                                            &device_ext, [(queue_family, 0.5)].iter().cloned()).expect("failed to create device");
    let queue = queues.next().unwrap();

    //Create swapchain
    let (mut swapchain, mut images) = {
        let caps = surface.capabilities(physical).expect("failed to get surface capabilities");
        dimensions = caps.current_extent.unwrap_or([1024,768]);
        let usage = caps.supported_usage_flags;
        let format = caps.supported_formats[0].0;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        vulkano::swapchain::Swapchain::new(device.clone(), surface.clone(),
                                           caps.min_image_count, format, dimensions, 1,
                                           usage, &queue, vulkano::swapchain::SurfaceTransform::Identity,
                                           alpha, vulkano::swapchain::PresentMode::Fifo, true, None)
            .expect("Failed to create swapchain")
    };

    //let filepath = "src/bencube/bencube";
    let filepath2 = "src/texturedmodel/Low";
    let filepath = "src/texturedmodel/Low";

    let sampler = vulkano::sampler::Sampler::new(device.clone(), vulkano::sampler::Filter::Linear,
                                                 vulkano::sampler::Filter::Linear, vulkano::sampler::MipmapMode::Nearest,
                                                 vulkano::sampler::SamplerAddressMode::Repeat, vulkano::sampler::SamplerAddressMode::Repeat,
                                                 vulkano::sampler::SamplerAddressMode::Repeat, 0.0, 1.0 , 0.0, 0.0).unwrap();

    let mut depth_buffer = vulkano::image::attachment::AttachmentImage::transient(device.clone(),dimensions, vulkano::format::D16Unorm).unwrap();

    let position = cgmath::Matrix4::from_translation(cgmath::vec3(0.0,0.0,0.0));
    let pos2 = cgmath::Matrix4::from_translation(cgmath::vec3(0.0,-1.0,0.0));

    let mut static_meshes:Vec<crate::support::object::StaticMesh> = Vec::new();
    let mut meshes = Vec::new();
    //TODO: test preformance of this and then implement better
    let queue1 = queue.clone();
    let device1 = device.clone();
    let load1 = thread::spawn(move || {
        support::object::StaticMesh::new(filepath, position, queue1, device1)
    });

    meshes.push(support::object::Mesh::new("Player".to_string(),filepath2,
                                           pos2, queue.clone(), device.clone()));
    static_meshes.push(load1.join().unwrap());

    //setup the camera data
    let mut proj = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_4),
                                       {dimensions[0] as f32 / dimensions[1] as f32 }, 0.01, 100.0);
    let mut camera = support::camera::Camera::new((0.0,0.0,-1.0), (0.0,0.0,0.0));
    let scale_amount:f32 = 0.25;
    let scale = cgmath::Matrix4::from_scale(scale_amount);

    let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<vs::ty::Data>
    ::new(device.clone(), vulkano::buffer::BufferUsage::all());

    //load vertex and fragment shader
    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    let renderpass = Arc::new(
        single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: vulkano::format::Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap()
    );
    //This will crash if the shader file isn't correct e.g. input not defined via the impl vertex macro
    //TODO:future proof this. Allow for more renderpasses for e.g. post processing
    let pipeline = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
        .vertex_input(support::FiveBuffersDefinition::new())
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .render_pass(vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());

    let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_,_>>>> = None;
    let mut recreate_swapchain = false;

    let mut previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;
    //defines programs start for delta time operations
    let start = std::time::Instant::now();

    let mut dynamic_state = vulkano::command_buffer::DynamicState {
        line_width: None,
        viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0 .. 1.0,
        }]),
        scissors: None,
    };
    //Defines the delta time variables
    let mut delta_time:f32;
    let mut elapsed_time:f64 = start.elapsed().as_secs() as f64;
    let mut previous_time:f64;
    //should be moved to camera object/entity
    #[allow(unused_mut)]
    let mut velocity = 1.0;
    let mut key_xy = support::camera::Keys {x:0.0, y:0.0};

    let mut count = 0;
    let mut global_focused = false;
    loop {
        previous_frame.cleanup_finished();
        //delta time calculations
        previous_time = elapsed_time.clone();
        elapsed_time = start.elapsed().as_secs() as f64 + (start.elapsed().subsec_millis() as f64/1000.0) as f64;
        delta_time = (elapsed_time-previous_time) as f32;
        //fps counter only print every 30 to save time
        count = count % 30;
        if count == 1{
            //println!{"FPS: {}", 1.0/delta_time};
        }

        count += 1;
        //should move to movement system
        if (key_xy.x != 0.0) || (key_xy.y != 0.0) {
            camera.move_by({
                let move_amount = key_xy.clone().get_normalized();
                let mut forward = math::normalize(camera.get_forward());
                forward = [(forward[0]*move_amount[0]*velocity*delta_time),
                    (forward[1]*move_amount[0]*velocity*delta_time),
                    (forward[2]*move_amount[0]*velocity*delta_time)];
                let mut side = math::normalize(camera.get_cross_dir());
                side = [(side[0]*move_amount[1]*velocity*delta_time),
                    (side[1]*move_amount[1]*velocity*delta_time),
                    (side[2]*move_amount[1]*velocity*delta_time)];
                [forward[0]+side[0],forward[1]+side[1],forward[2]+side[2]]
            });
            if camera.get_locked() {
                camera.set_lookat([meshes[0].position.w.x,
                    meshes[0].position.w.y,
                    meshes[0].position.w.z]);
            }
        }
        if recreate_swapchain {
            dimensions = surface.capabilities(physical)
                .expect("failed to get surface capabilities")
                .current_extent.unwrap_or([1024, 768]);
            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

            swapchain = new_swapchain;
            images = new_images;

            depth_buffer = vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, vulkano::format::D16Unorm).unwrap();

            framebuffers = None;
            //update the projection matrix to account for window resizing
            //TODO: move to camera entity
            proj = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), { dimensions[0] as f32 / dimensions[1] as f32 }, 0.01, 100.0);

            dynamic_state.viewports = Some(vec![vulkano::pipeline::viewport::Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]);
            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            framebuffers = Some(images.iter().map(|image| {
                Arc::new(vulkano::framebuffer::Framebuffer::start(renderpass.clone())
                    .add(image.clone()).unwrap()
                    .add(depth_buffer.clone()).unwrap()
                    .build().unwrap())
            }).collect::<Vec<_>>());
        }
        //TODO: Move to render system
        //This prepares the data for each object that needs to be rendered for the renderpass
        let mut static_uniform_buffers_subbuffers = Vec::new();
        for i in 0..static_meshes.len(){
            static_uniform_buffers_subbuffers.push({
                let uniform_data = vs::ty::Data {
                    world : (static_meshes[i].position * scale).into(),
                    view : camera.clone().get_view().into(),
                    proj : proj.into(),
                    camera : camera.get_position().into(),
                };
                uniform_buffer.next(uniform_data).unwrap()
            });
        }
        let mut uniform_buffers_subbuffers = Vec::new();
        for i in 0..meshes.len(){
            uniform_buffers_subbuffers.push({
                let uniform_data = vs::ty::Data {
                    world : (meshes[i].position * scale).into(),
                    view : camera.clone().get_view().into(),
                    proj : proj.into(),
                    camera : camera.get_position().into(),
                };
                uniform_buffer.next(uniform_data).unwrap()
            });
        }
        let mut static_sets = Vec::new();
        for i in 0..static_meshes.len(){
            static_sets.push(Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                .add_buffer(static_uniform_buffers_subbuffers[i].clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_base.clone(),sampler.clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_normal.clone(),sampler.clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_metallic.clone(),sampler.clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_roughness.clone(),sampler.clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_ao.clone(),sampler.clone()).unwrap()
                .build().unwrap()));
        }
        let mut sets = Vec::new();
        for i in 0..meshes.len(){
            sets.push(Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                .add_buffer(uniform_buffers_subbuffers[i].clone()).unwrap()
                .add_sampled_image(meshes[i].render_object.texture_base.clone(),sampler.clone()).unwrap()
                .add_sampled_image(meshes[i].render_object.texture_normal.clone(),sampler.clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_metallic.clone(),sampler.clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_roughness.clone(),sampler.clone()).unwrap()
                .add_sampled_image(static_meshes[i].render_object.texture_ao.clone(),sampler.clone()).unwrap()
                .build().unwrap()));
        }

        let (image_num, acquire_future) = match vulkano::swapchain::acquire_next_image(swapchain.clone(),
                                                                                       None) {
            Ok(r) => r,
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            },
            Err(err) => panic!("{:?}", err)
        };
        let command_buffer_begin = vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(
                framebuffers.as_ref().unwrap()[image_num].clone(), false,
                vec![
                    [0.0, 0.0, 1.0, 1.0].into(), //background colour definition
                    1f32.into()
                ]).unwrap();
        let command_buffer = {
            let mut temp = command_buffer_begin;
            if static_meshes.len() != 0 {
                for i in 0..static_meshes.len() {
                    temp = temp.draw_indexed(
                        pipeline.clone(),
                        &dynamic_state,
                        (static_meshes[i].render_object.vertex_buffer.clone(), static_meshes[i].render_object.normal_buffer.clone(), static_meshes[i].render_object.texture_coords.clone(), static_meshes[i].render_object.tangent_buffer.clone(), static_meshes[i].render_object.bitangent_buffer.clone()),    // , texcoords_buffer.clone() Need to fix the pipeline first to accept a third input
                        static_meshes[i].render_object.index_buffer.clone(), static_sets[i].clone(), ()).unwrap()

                }
            }
            for i in 0..meshes.len() {
                temp = temp.draw_indexed(
                    pipeline.clone(),
                    &dynamic_state,
                    (meshes[i].render_object.vertex_buffer.clone(), meshes[i].render_object.normal_buffer.clone(), meshes[i].render_object.texture_coords.clone(), meshes[i].render_object.tangent_buffer.clone(), meshes[i].render_object.bitangent_buffer.clone()),    // , texcoords_buffer.clone() Need to fix the pipeline first to accept a third input
                    meshes[i].render_object.index_buffer.clone(), sets[i].clone(), ()).unwrap()

            }
            temp.end_render_pass().unwrap()
                .build().unwrap()
        };
        let future = previous_frame.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame = Box::new(future) as Box<_>;
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
        }
        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => done = true,
                winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput { input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::W),state:key_state, ..},..},..}=> (
                    {
                        if winit::ElementState::Pressed == key_state {
                            if key_xy.x < 1.0 {
                                key_xy.x = key_xy.x+1.0;
                            }
                        } if key_state == winit::ElementState::Released {
                        if key_xy.x > -1.0{
                            key_xy.x = key_xy.x - 1.0;
                        }
                    }
                    }
                ),
                winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput { input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::S),state:key_state, ..},..},..}=> (
                    {
                        if winit::ElementState::Pressed == key_state {
                            if key_xy.x > -1.0 {
                                key_xy.x = key_xy.x-1.0;
                            }
                        } if key_state == winit::ElementState::Released {
                        if key_xy.x < 1.0{
                            key_xy.x = key_xy.x + 1.0;
                        }
                    }
                    }
                ),winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput { input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::A),state:key_state, ..},..},..}=> (
                    {
                        if winit::ElementState::Pressed == key_state {
                            if key_xy.y > -1.0 {
                                key_xy.y = key_xy.y-1.0;
                            }
                        } if key_state == winit::ElementState::Released {
                        if key_xy.y < 1.0{
                            key_xy.y = key_xy.y + 1.0;
                        }
                    }
                    }
                ),
                winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput { input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::D),state:key_state, ..},..},..}=> (
                    {
                        if winit::ElementState::Pressed == key_state {
                            if key_xy.y < 1.0 {
                                key_xy.y = key_xy.y+1.0;
                            }
                        } if key_state == winit::ElementState::Released {
                            if key_xy.y > -1.0{
                                key_xy.y = key_xy.y - 1.0;
                            }
                        }
                    }
                ),winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput { input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::Up),state:key_state, ..},..},..}=> (
                    {
                        if key_state == winit::ElementState::Released{
                            let temp = meshes[0].position.clone();
                            let temp = [temp.w.x, temp.w.y, temp.w.z];
                            let temp = [temp[0],temp[1]-0.1, temp[2]];
                            meshes[0].position.w.x = temp[0];
                            meshes[0].position.w.x = temp[1];
                            meshes[0].position.w.x = temp[2];
                        }
                    }
                ),
                winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput { input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::Down),state:key_state, ..},..},..}=> (
                    {
                        if key_state == winit::ElementState::Released{
                            let temp = meshes[0].position.clone();
                            let temp = [temp.w.x, temp.w.y, temp.w.z];
                            let temp = [temp[0],temp[1]+0.1, temp[2]];
                            meshes[0].position.w.x = temp[0];
                            meshes[0].position.w.x = temp[1];
                            meshes[0].position.w.x = temp[2];
                        }
                    }
                ),winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput { input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::F),state:winit::ElementState::Released, ..},..},..}=> (
                    {
                        surface.window().set_fullscreen(Some(surface.window().get_current_monitor()));
                    }
                ),winit::Event::WindowEvent {event: winit::WindowEvent::Focused(focused) ,..}=> (
                    {
                        global_focused = focused;
                    }
                ),winit::Event::WindowEvent {event: winit::WindowEvent::CursorMoved {position: pos, ..}, ..}=> (
                    {
                        let size = surface.window().get_inner_size().expect("didn't get any dimensions");
                        let centre = winit::dpi::LogicalPosition {x: size.width / 2.0, y: size.height / 2.0};
                        if false == camera.get_locked() {
                            let mut dir = camera.get_lookat();
                            let camerap = camera.get_position();
                            dir = [dir[0] - camerap[0],dir[1] - camerap[1],dir[2] - camerap[2]];
                            let xchange = (pos.x - centre.x) as f32;
                            let rot = cgmath::Matrix3::from_angle_y(cgmath::Rad(-(xchange / 960.0) * 3.14));

                            dir = math::vector_matrix_mul(dir, rot);
                            dir = [dir[0] + camerap[0], dir[1] + camerap[1], dir[2] + camerap[2]];
                            if xchange != 0.0{
                                camera.set_lookat(dir.clone());
                            }
                            let ychange = -(pos.y - centre.y) as f32;
                            let mut change = [0.0, (-ychange / 1000.0), 0.0];
                            let mut dir = camera.get_lookat();

                            dir = [dir[0]- camerap[0], dir[1] - camerap[1], dir[2] - camerap[2]];
                            change = math::normalize([dir[0] + change[0], dir[1] + change[1], dir[2] + change[2]]);
                            change = [change[0] + camerap[0], change[1] + camerap[1], change[2] + camerap[2]];
                            camera.set_lookat(change.clone());
                        }
                        if global_focused {
                            surface.window().set_cursor_position(centre).expect("Failed to centre cursor");
                        }
                    }
                ),winit::Event::WindowEvent{event: winit::WindowEvent::MouseInput {button: winit::MouseButton::Middle, state: winit::ElementState::Released, ..}, ..}=> (
                    {
                        camera.set_lookat([meshes[0].position.w.x, meshes[0].position.w.y, meshes[0].position.w.z]);
                        camera.locked = !camera.locked;
                    }
                ),
                _ => ()
            }
        });
        if done { return; }
    }
}
mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[path = "src/shader.vert"]

    #[allow(dead_code)]
    struct Dummy;
}
mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "src/shader.frag"]

    #[allow(dead_code)]
    struct Dummy;
}