use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer };
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::image::attachment::AttachmentImage;
use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::vertex::TwoBuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use vulkano_win::VkSurfaceBuild;

use winit::{EventsLoop, Window, WindowBuilder, Event, WindowEvent};

use std::sync::Arc;
use std::iter;
use std::time::Instant;

pub type RenderParams = fn() -> (Vec<Vertex3>, Vec<Colour4>, Vec<u16>);

pub fn run(update_values: RenderParams) {
    run_internal(update_values, false);
}

pub fn run_show_fps(update_values: RenderParams) {
    run_internal(update_values, true);
}

fn run_internal(update_values: RenderParams, show_fps: bool) {
    println!("Beginning Vulkan setup...");
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).unwrap()
    };

    // We then choose which physical device to use.
    //
    // In a real application, there are three things to take into consideration:
    //
    // - Some devices may not support some of the optional features that may be required by your
    //   application. You should filter out the devices that don't support your app.
    //
    // - Not all devices can draw to a certain surface. Once you create your window, you have to
    //   choose a device that is capable of drawing to it.
    //
    // - You probably want to leave the choice between the remaining devices to the user.
    //
    let mut physical_devices = PhysicalDevice::enumerate(&instance);

    for device in physical_devices.clone() {
        println!("Found device: {} (type: {:?})", device.name(), device.ty());
    }
    let physical = physical_devices.next().unwrap();
    // Some debug info.
    println!("Using {}.", physical.name());


    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
    let window = surface.window();

    let queue_family = physical.queue_families().find(|&q| {
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    }).unwrap();

    let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
    let (device, mut queues) = Device::new(physical, physical.supported_features(), &device_ext,
                                           [(queue_family, 0.5)].iter().cloned()).unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();
        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
        } else {
            return;
        };

        Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format,
                       initial_dimensions, 1, usage, &queue, SurfaceTransform::Identity, alpha,
                       PresentMode::Fifo, true, None).unwrap()

    };

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
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
                format: Format::D16Unorm,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    ).unwrap());

    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone());
    let mut recreate_swapchain = false;

    let mut previous_frame_end = Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>;

    let mut t0 = Instant::now();
    let mut updates = 0;
    let fps_freq = 100;
    loop {
        if show_fps {
            previous_frame_end.cleanup_finished();
            updates += 1;
            if updates % fps_freq == 0 {
                let t = Instant::now();
                let ms = t.checked_duration_since(t0).unwrap().as_millis() as f32 / fps_freq as f32;
                let fps = 1000.0 / ms;
                println!("{} fps", fps);
                t0 = Instant::now();
            }
        }

        // Whenever the window resizes we need to recreate everything dependent on the window size.
        // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
        if recreate_swapchain {
            // Get the new dimensions of the window.
            let (phys_dims, log_dims) = if let Some(dimensions) = window.get_inner_size() {
                let log: (u32, u32) = dimensions.into();
                let phys: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                ([phys.0, phys.1], [log.0, log.1])
            } else {
                return;
            };

            println!("Physical window dimensions: {:?}\nLogical window dimensions: {:?}", phys_dims, log_dims);

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(phys_dims) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err)
            };

            swapchain = new_swapchain;
            let (new_pipeline, new_framebuffers) = window_size_dependent_setup(device.clone(), &vs, &fs, &new_images, render_pass.clone());
            pipeline = new_pipeline;
            framebuffers = new_framebuffers;

            recreate_swapchain = false;
        }

        let (image_num, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            },
            Err(err) => panic!("{:?}", err)
        };

        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into(), 1f32.into()];

        let (vertices, colours, indices) = update_values();

        let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), vertices.iter().cloned()).unwrap();
        let colour_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), colours.iter().cloned()).unwrap();
        let index_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), indices.iter().cloned()).unwrap();

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(framebuffers[image_num].clone(), false, clear_values).unwrap()
            .draw_indexed(
                pipeline.clone(),
                &DynamicState::none(),
                vec!(vertex_buffer.clone(), colour_buffer.clone()),
                index_buffer.clone(), (), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame_end = Box::new(future) as Box<_>;
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => done = true,
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
                _ => ()
            }
        });
        if done { return; }
    }
}

fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>
) -> (Arc<dyn GraphicsPipelineAbstract + Send + Sync>, Vec<Arc<dyn FramebufferAbstract + Send + Sync>>) {
    let dimensions = images[0].dimensions();
    let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

    let framebuffers = images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>();

    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input(TwoBuffersDefinition::<Vertex3, Colour4>::new())
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .viewports(iter::once(Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0 .. 1.0,
        }))
        .fragment_shader(fs.main_entry_point(), ())
        .blend_alpha_blending()
        .depth_stencil_simple_depth()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());

    (pipeline, framebuffers)
}

#[derive(Default, Debug, Clone)]
pub struct Vertex3 { pub position: (f32, f32, f32) }
vulkano::impl_vertex!(Vertex3, position);

#[derive(Default, Debug, Clone)]
pub struct Colour4 { pub colour: (f32, f32, f32, f32) }
vulkano::impl_vertex!(Colour4, colour);

mod vs {
    vulkano_shaders::shader!{
            ty: "vertex",
            src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 colour;

layout(location = 0) out vec4 fragColour;

void main() {
    gl_Position = vec4(position, 1.0);
    fragColour = colour;
}"
        }
}

mod fs {
    vulkano_shaders::shader!{
            ty: "fragment",
            src: "
#version 450

layout(location = 0) in vec4 fragColour;
layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(fragColour);
}
"
        }
}