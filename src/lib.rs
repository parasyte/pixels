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

use std::cell::RefCell;
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;

use vk_shader_macros::include_glsl;
pub use wgpu;
use wgpu::TextureView;

mod render_pass;
pub use render_pass::RenderPass;

// Type aliases for RenderPass
type RPObject = Box<dyn RenderPass>;
type RPDevice = Rc<wgpu::Device>;
type RPQueue = Rc<RefCell<wgpu::Queue>>;
type RenderPassFactory = Box<dyn Fn(RPDevice, RPQueue, &TextureView) -> RPObject>;

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
    // WGPU state
    device: Rc<wgpu::Device>,
    queue: Rc<RefCell<wgpu::Queue>>,
    swap_chain: wgpu::SwapChain,

    // List of render passes
    renderers: Vec<RPObject>,
}

/// A builder to help create customized pixel buffers.
pub struct PixelsBuilder<'a> {
    request_adapter_options: wgpu::RequestAdapterOptions,
    device_descriptor: wgpu::DeviceDescriptor,
    width: u32,
    height: u32,
    pixel_aspect_ratio: f64,
    surface_texture: SurfaceTexture<'a>,
    texture_format: wgpu::TextureFormat,
    renderer_factories: Vec<RenderPassFactory>,
}

/// Renderer implements RenderPass.
#[derive(Debug)]
struct Renderer {
    device: Rc<wgpu::Device>,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    texture_extent: wgpu::Extent3d,
    texture_format_size: u32,
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
    pub fn new(width: u32, height: u32, surface_texture: SurfaceTexture) -> Result<Pixels, Error> {
        PixelsBuilder::new(width, height, surface_texture).build()
    }

    // TODO: Support resize

    /// Draw this pixel buffer to the configured [`SurfaceTexture`].
    ///
    /// This executes all render passes in sequence. See [`RenderPass`].
    ///
    /// # Arguments
    ///
    /// * `texels` - Byte slice of texture pixels (AKA texels) to draw. The texture format can be
    /// configured with [`PixelsBuilder`].
    pub fn render(&mut self, texels: &[u8]) {
        // TODO: Center frame buffer in surface
        let frame = self.swap_chain.get_next_texture();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        // Execute all render passes
        for renderer in self.renderers.iter() {
            // TODO: Create a texture chain so that each pass receives the texture drawn by the previous
            renderer.render_pass(&mut encoder, &frame.view, texels);
        }

        self.queue.borrow_mut().submit(&[encoder.finish()]);
    }
}

impl RenderPass for Renderer {
    fn update_bindings(&mut self, _texture_view: &TextureView) {}

    fn render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &TextureView,
        texels: &[u8],
    ) {
        // Update the pixel buffer texture view
        let buffer = self
            .device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&texels);
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                row_pitch: self.texture_extent.width * self.texture_format_size,
                image_height: self.texture_extent.height,
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            self.texture_extent,
        );

        // Draw the updated texture to the render target
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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
    /// struct MyRenderPass {
    ///     // ...
    /// };
    ///
    /// impl pixels::RenderPass for MyRenderPass {
    ///     // ...
    /// # fn update_bindings(&mut self, _: &wgpu::TextureView) {}
    /// # fn render_pass(&self, _: &mut wgpu::CommandEncoder, _: &wgpu::TextureView, _: &[u8]) {}
    /// }
    ///
    /// let fb = PixelsBuilder::new(256, 240, surface_texture)
    ///     .pixel_aspect_ratio(8.0 / 7.0)
    ///     .add_render_pass(|device, queue, texture| {
    ///         // Create reources for MyRenderPass here
    ///         Box::new(MyRenderPass {
    ///             // ...
    ///         })
    ///     })
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
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            renderer_factories: Vec::new(),
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

    /// Set the texture format.
    ///
    /// The default value is [`wgpu::TextureFormat::Rgba8UnormSrgb`], which is 4 unsigned bytes in
    /// `RGBA` order using the SRGB color space. This is typically what you want when you are
    /// working with color values from popular image editing tools or web apps.
    pub fn texture_format(mut self, texture_format: wgpu::TextureFormat) -> PixelsBuilder<'a> {
        self.texture_format = texture_format;
        self
    }

    /// Add a render pass.
    ///
    /// Render passes are executed in the order they are added.
    ///
    /// # Factory Arguments
    ///
    /// * `device` - A reference-counted [`wgpu::Device`] which allows you to create GPU resources.
    /// * `queue` - A reference-counted [`wgpu::Queue`] which can execute command buffers.
    /// * `texture` - A [`wgpu::TextureView`] reference that is used as the texture input for the
    /// render pass.
    pub fn add_render_pass(
        mut self,
        factory: impl Fn(RPDevice, RPQueue, &TextureView) -> RPObject + 'static,
    ) -> PixelsBuilder<'a> {
        self.renderer_factories.push(Box::new(factory));
        self
    }

    /// Create a pixel buffer from the options builder.
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] cannot be found.
    pub fn build(self) -> Result<Pixels, Error> {
        // TODO: Use `options.pixel_aspect_ratio` to stretch the scaled texture

        let adapter =
            wgpu::Adapter::request(&self.request_adapter_options).ok_or(Error::AdapterNotFound)?;
        let (device, queue) = adapter.request_device(&self.device_descriptor);
        let device = Rc::new(device);
        let queue = Rc::new(RefCell::new(queue));

        let vs_module = device.create_shader_module(include_glsl!("shaders/shader.vert"));
        let fs_module = device.create_shader_module(include_glsl!("shaders/shader.frag"));

        // The rest of this is technically a fixed-function pipeline... For now!

        // Create a texture
        let width = self.width;
        let height = self.height;
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_default_view();
        let texture_format_size = get_texture_format_size(self.texture_format);

        // Create a texture sampler with nearest neighbor
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create pipeline
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

        // Create swap chain
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

        // Create a renderer that impls `RenderPass`
        let renderer = Renderer {
            device: device.clone(),
            bind_group,
            render_pipeline,
            texture,
            texture_extent,
            texture_format_size,
        };

        let mut renderers: Vec<Box<dyn RenderPass>> = vec![Box::new(renderer)];

        // Create all render passes
        renderers.extend(self.renderer_factories.iter().map(|f| {
            // TODO: Create a texture chain so that each pass recieves the texture drawn by the previous
            f(device.clone(), queue.clone(), &texture_view)
        }));

        Ok(Pixels {
            device,
            queue,
            swap_chain,
            renderers,
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

fn get_texture_format_size(texture_format: wgpu::TextureFormat) -> u32 {
    match texture_format {
        // 8-bit formats
        wgpu::TextureFormat::R8Unorm
        | wgpu::TextureFormat::R8Snorm
        | wgpu::TextureFormat::R8Uint
        | wgpu::TextureFormat::R8Sint => 1,

        // 16-bit formats
        wgpu::TextureFormat::R16Unorm
        | wgpu::TextureFormat::R16Snorm
        | wgpu::TextureFormat::R16Uint
        | wgpu::TextureFormat::R16Sint
        | wgpu::TextureFormat::R16Float
        | wgpu::TextureFormat::Rg8Unorm
        | wgpu::TextureFormat::Rg8Snorm
        | wgpu::TextureFormat::Rg8Uint
        | wgpu::TextureFormat::Rg8Sint => 2,

        // 32-bit formats
        wgpu::TextureFormat::R32Uint
        | wgpu::TextureFormat::R32Sint
        | wgpu::TextureFormat::R32Float
        | wgpu::TextureFormat::Rg16Unorm
        | wgpu::TextureFormat::Rg16Snorm
        | wgpu::TextureFormat::Rg16Uint
        | wgpu::TextureFormat::Rg16Sint
        | wgpu::TextureFormat::Rg16Float
        | wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Rgba8Snorm
        | wgpu::TextureFormat::Rgba8Uint
        | wgpu::TextureFormat::Rgba8Sint
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb
        | wgpu::TextureFormat::Rgb10a2Unorm
        | wgpu::TextureFormat::Rg11b10Float
        | wgpu::TextureFormat::Depth32Float
        | wgpu::TextureFormat::Depth24Plus
        | wgpu::TextureFormat::Depth24PlusStencil8 => 4,

        // 64-bit formats
        wgpu::TextureFormat::Rg32Uint
        | wgpu::TextureFormat::Rg32Sint
        | wgpu::TextureFormat::Rg32Float
        | wgpu::TextureFormat::Rgba16Unorm
        | wgpu::TextureFormat::Rgba16Snorm
        | wgpu::TextureFormat::Rgba16Uint
        | wgpu::TextureFormat::Rgba16Sint
        | wgpu::TextureFormat::Rgba16Float => 8,

        // 128-bit formats
        wgpu::TextureFormat::Rgba32Uint
        | wgpu::TextureFormat::Rgba32Sint
        | wgpu::TextureFormat::Rgba32Float => 16,
    }
}
