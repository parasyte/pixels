//! A tiny library providing a GPU-powered pixel pixel buffer.
//!
//! [`Pixels`] represents a 2D pixel buffer with an explicit image resolution, making it ideal for
//! prototyping simple pixel-based games, animations, and emulators. The pixel buffer is rendered
//! entirely on the GPU, allowing developers to easily incorporate special effects with shaders and
//! a customizable pipeline.
//!
//! The GPU interface is offered by [`wgpu`](https://crates.io/crates/wgpu), and is re-exported for
//! your convenience. Use a windowing framework or context manager of your choice;
//! [`winit`](https://crates.io/crates/winit) is a good place to start.

use std::error::Error as StdError;
use std::fmt;

use vk_shader_macros::include_glsl;
pub use wgpu;

mod render_pass;
pub use render_pass::RenderPass;

/// A logical texture for a window surface.
#[derive(Debug)]
pub struct SurfaceTexture<'a> {
    surface: &'a wgpu::Surface,
    width: u32,
    height: u32,
}

/// Represents a 2D pixel buffer with an explicit image resolution.
///
/// See [`PixelsBuilder`] for building a customized pixel buffer.
#[derive(Debug)]
pub struct Pixels {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    swap_chain: wgpu::SwapChain,
}

/// A builder to help create customized pixel buffers.
#[derive(Debug)]
pub struct PixelsBuilder<'a> {
    request_adapter_options: wgpu::RequestAdapterOptions,
    device_descriptor: wgpu::DeviceDescriptor,
    width: u32,
    height: u32,
    pixel_aspect_ratio: f64,
    surface_texture: SurfaceTexture<'a>,
}

/// Renderer implements RenderPass.
#[derive(Debug)]
struct Renderer {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

/// All the ways in which creating a pixel buffer can fail.
#[derive(Debug)]
pub enum Error {
    /// No suitable [`wgpu::Adapter`] found
    AdapterNotFound,
}

impl<'a> SurfaceTexture<'a> {
    /// Create a logical texture for a window surface.
    ///
    /// It is recommended (but not required) that the `width` and `height` are equivalent to the
    /// physical dimentions of the `surface`. E.g. scaled by the HiDPI factor.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pixels::SurfaceTexture;
    /// use wgpu::Surface;
    /// use winit::event_loop::EventLoop;
    /// use winit::window::Window;
    ///
    /// let event_loop = EventLoop::new();
    /// let window = Window::new(&event_loop).unwrap();
    /// let surface = Surface::create(&window);
    /// let size = window.inner_size().to_physical(window.hidpi_factor());
    ///
    /// let width = size.width as u32;
    /// let height = size.height as u32;
    ///
    /// let surface_texture = SurfaceTexture::new(width, height, &surface);
    /// # Ok::<(), pixels::Error>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when `width` or `height` are 0.
    pub fn new(width: u32, height: u32, surface: &'a wgpu::Surface) -> SurfaceTexture<'a> {
        assert!(width > 0);
        assert!(height > 0);

        SurfaceTexture {
            surface,
            width,
            height,
        }
    }
}

impl Pixels {
    /// Create a pixel buffer instance with default options.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, &surface);
    /// let fb = Pixels::new(320, 240, surface_texture)?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] cannot be found.
    ///
    /// # Panics
    ///
    /// Panics when `width` or `height` are 0.
    pub fn new<'a>(
        width: u32,
        height: u32,
        surface_texture: SurfaceTexture<'a>,
    ) -> Result<Pixels, Error> {
        PixelsBuilder::new(width, height, surface_texture).build()
    }

    // TODO: Support resize

    /// Draw this pixel buffer to the configured [`SurfaceTexture`].
    pub fn render(&mut self) {
        // TODO: Center frame buffer in surface
        let frame = self.swap_chain.get_next_texture();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        // TODO: Run all render passes in a loop
        self.renderer.render_pass(&mut encoder, &frame.view);

        self.queue.submit(&[encoder.finish()]);
    }
}

impl RenderPass for Renderer {
    fn update_bindings(&mut self, _texture_view: &wgpu::TextureView) {}

    fn render_pass(&self, encoder: &mut wgpu::CommandEncoder, render_target: &wgpu::TextureView) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::GREEN,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}

impl<'a> PixelsBuilder<'a> {
    /// Create a builder that can be finalized into a [`Pixels`] pixel buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pixels::PixelsBuilder;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, &surface);
    /// let fb = PixelsBuilder::new(256, 240, surface_texture)
    ///     .pixel_aspect_ratio(8.0 / 7.0)
    /// #   // TODO: demonstrate adding a render pass here
    /// #   //.render_pass(...)
    ///     .build()?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when `width` or `height` are 0.
    pub fn new(width: u32, height: u32, surface_texture: SurfaceTexture<'a>) -> PixelsBuilder<'a> {
        assert!(width > 0);
        assert!(height > 0);

        PixelsBuilder {
            request_adapter_options: wgpu::RequestAdapterOptions::default(),
            device_descriptor: wgpu::DeviceDescriptor::default(),
            width,
            height,
            pixel_aspect_ratio: 1.0,
            surface_texture,
        }
    }

    /// Add options for requesting a [`wgpu::Adapter`].
    pub fn request_adapter_options(
        mut self,
        request_adapter_options: wgpu::RequestAdapterOptions,
    ) -> PixelsBuilder<'a> {
        self.request_adapter_options = request_adapter_options;
        self
    }

    /// Add options for requesting a [`wgpu::Device`].
    pub fn device_descriptor(
        mut self,
        device_descriptor: wgpu::DeviceDescriptor,
    ) -> PixelsBuilder<'a> {
        self.device_descriptor = device_descriptor;
        self
    }

    /// Set the pixel aspect ratio to simulate non-square pixels.
    ///
    /// This setting enables a render pass that horizontally scales the pixel buffer by the given
    /// factor.
    ///
    /// E.g. set this to `8.0 / 7.0` for an 8:7 pixel aspect ratio.
    pub fn pixel_aspect_ratio(mut self, pixel_aspect_ratio: f64) -> PixelsBuilder<'a> {
        self.pixel_aspect_ratio = pixel_aspect_ratio;
        self
    }

    /// Create a pixel buffer from the options builder.
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] cannot be found.
    pub fn build(self) -> Result<Pixels, Error> {
        // TODO: Create a texture with the dimensions specified in `options`
        // TODO: Use `options.pixel_aspect_ratio` to stretch the scaled texture

        let adapter =
            wgpu::Adapter::request(&self.request_adapter_options).ok_or(Error::AdapterNotFound)?;
        let (device, queue) = adapter.request_device(&self.device_descriptor);

        let vs_module = device.create_shader_module(include_glsl!("shaders/shader.vert"));
        let fs_module = device.create_shader_module(include_glsl!("shaders/shader.frag"));

        // The rest of this is technically a fixed-function pipeline... For now!
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &[] });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let swap_chain = device.create_swap_chain(
            self.surface_texture.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: self.surface_texture.width,
                height: self.surface_texture.height,
                present_mode: wgpu::PresentMode::Vsync,
            },
        );

        let renderer = Renderer {
            bind_group,
            render_pipeline,
        };

        Ok(Pixels {
            device,
            queue,
            renderer,
            swap_chain,
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::AdapterNotFound => "No suitable Adapter found",
        }
    }
}
