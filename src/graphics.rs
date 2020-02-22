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
use std::error::Error;

pub enum Vertex {
    Vert2(f32, f32),
    Vert3(f32, f32, f32)
}

impl Vertex {
    pub fn from_v2(vertices: &[(f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(x, y)| Self::Vert2(*x, *y)).collect()
    }
    pub fn from_v3(vertices: &[(f32, f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(x, y, z)| Self::Vert3(*x, *y, *z)).collect()
    }
}

pub enum Colour {
    Rgb(f32, f32, f32),
    Rgba(f32, f32, f32, f32)
}

impl Colour {
    pub fn from_rgb(vertices: &[(f32, f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(r, g, b)| Self::Rgb(*r, *g, *b)).collect()
    }
    pub fn from_rgba(vertices: &[(f32, f32, f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(r, g, b, a)| Self::Rgba(*r, *g, *b, *a)).collect()
    }
}

pub type Index = u16;

pub struct RuntimeParams {
    pub window_width: u32,
    pub window_height: u32,
}

pub type RenderParams = (Vec<Vertex>, Vec<Colour>, Vec<Index>);
type VertexProducer = dyn Fn(RuntimeParams) -> RenderParams;

pub trait Backend {
    fn new() -> Result<Self, Box<dyn Error>> where Self: std::marker::Sized;
    fn show_fps(self) -> Self;
    fn run(self, update_values: &VertexProducer) -> Result<(), Box<dyn Error>>;
}

pub fn new() -> Result<backends::VulkanBackend, Box<dyn Error>> {
    backends::VulkanBackend::new()
}

pub mod backends {
    use super::*;
    use vulkano::device::Queue;
    use vulkano::swapchain::Surface;

    pub struct VulkanBackend {
        show_fps: bool,
        device: Arc<Device>,
        vs: vs::Shader,
        fs: fs::Shader,
        swapchain: Arc<Swapchain<winit::Window>>,
        images: Vec<Arc<SwapchainImage<Window>>>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        surface: Arc<Surface<winit::Window>>,
        queue: Arc<Queue>,
        events_loop: EventsLoop,
        phys_dims: [u32; 2],
        log_dims: [u32; 2],
    }

    impl VulkanBackend {
        fn window_size_dependent_setup(&self) -> (Arc<(dyn GraphicsPipelineAbstract + Send + Sync)>, Vec<Arc<dyn FramebufferAbstract + Send + Sync>>) {
            let dimensions = self.images[0].dimensions();
            let depth_buffer = AttachmentImage::transient(self.device.clone(), dimensions, Format::D16Unorm).unwrap();

            let framebuffers = self.images.iter().map(|image| {
                Arc::new(
                    Framebuffer::start(self.render_pass.clone())
                        .add(image.clone()).unwrap()
                        .add(depth_buffer.clone()).unwrap()
                        .build().unwrap()
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            }).collect::<Vec<_>>();

            let pipeline = Arc::new(GraphicsPipeline::start()
                .vertex_input(TwoBuffersDefinition::<VkVertex, VkColour>::new())
                .vertex_shader(self.vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(self.fs.main_entry_point(), ())
                .blend_alpha_blending()
                .depth_stencil_simple_depth()
                .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
                .build(self.device.clone())
                .unwrap());

            (pipeline, framebuffers)
        }

        fn convert_vertex(&self, vert: Vertex) -> VkVertex {
            let mut position = match vert {
                Vertex::Vert2(x, y) => (x, y, 0.0),
                Vertex::Vert3(x, y, z) => (x, y, z)
            };

            position.0 /= self.log_dims[0] as f32;
            position.1 /= self.log_dims[1] as f32;
            position.0 -= 0.5;
            position.1 -= 0.5;

            VkVertex { position }
        }
    }

    impl Backend for VulkanBackend {
        fn new() -> Result<Self, Box<dyn Error>> {
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


            let events_loop = EventsLoop::new();
            let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
            let window = surface.window();

            let queue_family = physical.queue_families().find(|&q| {
                q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
            }).unwrap();

            let device_ext = DeviceExtensions { khr_swapchain: true, ..DeviceExtensions::none() };
            let (device, mut queues) = Device::new(physical, physical.supported_features(), &device_ext,
                                                   [(queue_family, 0.5)].iter().cloned()).unwrap();

            let queue = queues.next().unwrap();

            let (phys_dims, log_dims) = if let Some(dimensions) = window.get_inner_size() {
                let log: (u32, u32) = dimensions.into();
                let phys: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                ([phys.0, phys.1], [log.0, log.1])
            } else {
                return Err("Failed to load window dimensions".into());
            };

            let (swapchain, images) = {
                let caps = surface.capabilities(physical).unwrap();
                let usage = caps.supported_usage_flags;
                let alpha = caps.supported_composite_alpha.iter().next().unwrap();
                let format = caps.supported_formats[0].0;

                Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format,
                               phys_dims, 1, usage, &queue, SurfaceTransform::Identity, alpha,
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

            let show_fps = false;
            let images = images.to_vec();

            Ok(Self {
                show_fps,
                device,
                vs,
                fs,
                images,
                render_pass,
                swapchain,
                surface,
                queue,
                events_loop,
                phys_dims,
                log_dims
            })
        }

        fn show_fps(mut self) -> Self {
            self.show_fps = true;
            self
        }

        fn run(mut self, update_values: &VertexProducer) -> Result<(), Box<dyn Error>> {
            let (mut pipeline, mut framebuffers) = self.window_size_dependent_setup();
            let mut recreate_swapchain = false;
            let window = self.surface.window();

            let mut previous_frame_end = Box::new(sync::now(self.device.clone())) as Box<dyn GpuFuture>;

            let mut t0 = Instant::now();
            let mut updates = 0;
            let fps_freq = 100;
            loop {
                if self.show_fps {
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
                        return Err("Failed to load window dimensions".into());
                    };
                    self.phys_dims = phys_dims;
                    self.log_dims = log_dims;

//                    println!("Physical window dimensions: {:?}\nLogical window dimensions: {:?}", phys_dims, log_dims);

                    let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(phys_dims) {
                        Ok(r) => r,
                        Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                        Err(err) => panic!("{:?}", err)
                    };

                    self.swapchain = new_swapchain;
                    self.images = new_images.to_vec();

                    let (new_pipeline, new_framebuffers) = self.window_size_dependent_setup();
                    pipeline = new_pipeline;
                    framebuffers = new_framebuffers;

                    recreate_swapchain = false;
                }

                let (image_num, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        continue;
                    },
                    Err(err) => panic!("{:?}", err)
                };

                let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into(), 1f32.into()];

                let (vertices, colours, indices) = update_values(RuntimeParams {
                    window_width: self.log_dims[0],
                    window_height: self.log_dims[1]
                });
                let vertices: Vec<VkVertex> = vertices.into_iter().map(|vert| self.convert_vertex(vert)).collect();
                let colours: Vec<VkColour> = colours.into_iter().map(|col| VkColour::from(col)).collect();

                let vertex_buffer = CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), vertices.iter().cloned()).unwrap();
                let colour_buffer = CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), colours.iter().cloned()).unwrap();
                let index_buffer = CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), indices.iter().cloned()).unwrap();

                let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family()).unwrap()
                    .begin_render_pass(framebuffers[image_num].clone(), false, clear_values).unwrap()
                    .draw_indexed(
                        pipeline.clone(),
                        &DynamicState::none(),
                        vec!(vertex_buffer.clone(), colour_buffer.clone()),
                        index_buffer.clone(), (), ()).unwrap()
                    .end_render_pass().unwrap()
                    .build().unwrap();

                let future = previous_frame_end.join(acquire_future)
                    .then_execute(self.queue.clone(), command_buffer).unwrap()
                    .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Box::new(future) as Box<_>;
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Box::new(sync::now(self.device.clone())) as Box<_>;
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        previous_frame_end = Box::new(sync::now(self.device.clone())) as Box<_>;
                    }
                }

                let mut done = false;
                self.events_loop.poll_events(|ev| {
                    match ev {
                        Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => done = true,
                        Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
                        _ => ()
                    }
                });
                if done { return Ok(()); }
            }
        }
    }

    #[derive(Default, Debug, Clone)]
    struct VkVertex { position: (f32, f32, f32) }
    vulkano::impl_vertex!(VkVertex, position);


    #[derive(Default, Debug, Clone)]
    struct VkColour { colour: (f32, f32, f32, f32) }
    vulkano::impl_vertex!(VkColour, colour);

    impl From<Colour> for VkColour {
        fn from(col: Colour) -> Self {
            let colour = match col {
                Colour::Rgb(r, g, b) => (r, g, b, 1.0),
                Colour::Rgba(r, g, b, a) => (r, g, b, a)
            };
            Self { colour }
        }
    }

    mod vs {
        vulkano_shaders::shader! {
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
        vulkano_shaders::shader! {
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
}