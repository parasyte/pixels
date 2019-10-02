//! A tiny library providing a GPU-powered pixel frame buffer.
//!
//! `Pixels` represents a 2D frame buffer with an explicit image resolution,
//! making it ideal for prototyping simple pixel-based games, animations, and
//! emulators. The frame buffer is rendered entirely on the GPU, allowing
//! developers to easily incorporate special effects with shaders and a
//! customizable pipeline.
//!
//! The GPU interface is offered by [`wgpu`](https://crates.io/crates/wgpu), and
//! is re-exported for your convenience. Use a windowing framework or context
//! manager of your choice; [`winit`](https://crates.io/crates/winit) is a good
//! place to start.

use std::error::Error as StdError;
use std::fmt;

use vk_shader_macros::include_glsl;
pub use wgpu;

/// Represents a 2D frame buffer with an explicit image resolution.
#[derive(Debug)]
pub struct Pixels {
    bind_group: wgpu::BindGroup,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    swap_chain: wgpu::SwapChain,
}

/// A builder to help create customized frame buffers.
#[derive(Debug)]
pub struct PixelsOptions {
    request_adapter_options: wgpu::RequestAdapterOptions,
    device_descriptor: wgpu::DeviceDescriptor,
}

/// All the ways in which creating a frame buffer can fail.
#[derive(Debug)]
pub enum Error {
    /// No suitable Adapter found
    AdapterNotFound,
}

/// # Examples
///
/// ```no_run
/// # use pixels::Pixels;
/// # struct RWH();
/// # unsafe impl raw_window_handle::HasRawWindowHandle for RWH {
/// #   fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
/// #     #[cfg(target_os = "macos")]
/// #     return raw_window_handle::RawWindowHandle::MacOS(
/// #       raw_window_handle::macos::MacOSHandle::empty()
/// #     );
/// #     #[cfg(any(
/// #       target_os = "linux",
/// #       target_os = "dragonfly",
/// #       target_os = "freebsd",
/// #       target_os = "netbsd",
/// #       target_os = "openbsd"))]
/// #     return raw_window_handle::RawWindowHandle::X11(
/// #       raw_window_handle::unix::X11Handle::empty()
/// #     );
/// #     #[cfg(target_os = "windows")]
/// #     return raw_window_handle::RawWindowHandle::Windows(
/// #       raw_window_handle::windows::WindowsHandle::empty()
/// #     );
/// #     #[cfg(target_os = "ios")]
/// #     return raw_window_handle::RawWindowHandle::IOS(
/// #       raw_window_handle::ios::IOSHandle::empty()
/// #     );
/// #   }
/// # }
/// # let surface = wgpu::Surface::create(&RWH());
/// let fb = Pixels::new(320, 240, &surface)?;
/// # Ok::<(), pixels::Error>(())
/// ```
impl Pixels {
    /// Create a frame buffer instance with default options.
    ///
    /// # Errors
    ///
    /// Returns an error when a `wgpu::Adapter` cannot be found.
    pub fn new(width: u32, height: u32, surface: &wgpu::Surface) -> Result<Pixels, Error> {
        Pixels::new_with_options(width, height, surface, PixelsOptions::new())
    }

    /// Create a frame buffer instance with the given options.
    ///
    /// # Errors
    ///
    /// Returns an error when a `wgpu::Adapter` cannot be found or shaders
    /// are invalid SPIR-V.
    pub fn new_with_options(
        width: u32,
        height: u32,
        surface: &wgpu::Surface,
        options: PixelsOptions,
    ) -> Result<Pixels, Error> {
        let adapter = wgpu::Adapter::request(&options.request_adapter_options)
            .ok_or(Error::AdapterNotFound)?;
        let (device, queue) = adapter.request_device(&options.device_descriptor);

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
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width,
                height,
                present_mode: wgpu::PresentMode::Vsync,
            },
        );

        Ok(Pixels {
            bind_group,
            device,
            queue,
            render_pipeline,
            swap_chain,
        })
    }

    // TODO: Support resize

    /// Draw this frame buffer to the configured `wgpu::Surface`.
    pub fn render(&mut self) {
        // TODO: Center frame buffer in surface
        let frame = self.swap_chain.get_next_texture();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
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

        self.queue.submit(&[encoder.finish()]);
    }
}

/// # Examples
///
/// ```no_run
/// # use pixels::PixelsOptions;
/// # struct RWH();
/// # unsafe impl raw_window_handle::HasRawWindowHandle for RWH {
/// #   fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
/// #     #[cfg(target_os = "macos")]
/// #     return raw_window_handle::RawWindowHandle::MacOS(
/// #       raw_window_handle::macos::MacOSHandle::empty()
/// #     );
/// #     #[cfg(any(
/// #       target_os = "linux",
/// #       target_os = "dragonfly",
/// #       target_os = "freebsd",
/// #       target_os = "netbsd",
/// #       target_os = "openbsd"))]
/// #     return raw_window_handle::RawWindowHandle::X11(
/// #       raw_window_handle::unix::X11Handle::empty()
/// #     );
/// #     #[cfg(target_os = "windows")]
/// #     return raw_window_handle::RawWindowHandle::Windows(
/// #       raw_window_handle::windows::WindowsHandle::empty()
/// #     );
/// #     #[cfg(target_os = "ios")]
/// #     return raw_window_handle::RawWindowHandle::IOS(
/// #       raw_window_handle::ios::IOSHandle::empty()
/// #     );
/// #   }
/// # }
/// # fn main() -> Result<(), pixels::Error> {
/// # let surface = wgpu::Surface::create(&RWH());
/// let fb = PixelsOptions::new()
///     .build(320, 240, &surface)?;
/// # Ok(())
/// # }
/// ```
impl PixelsOptions {
    /// Create a builder that can be finalized into a frame buffer instance.
    pub fn new() -> PixelsOptions {
        PixelsOptions::default()
    }

    /// Add options for requesting a `wgpu::Adapter`.
    pub fn request_adapter_options(mut self, rao: wgpu::RequestAdapterOptions) -> PixelsOptions {
        self.request_adapter_options = rao;
        self
    }

    /// Add options for requesting a `wgpu::Device`.
    pub fn device_descriptor(mut self, dd: wgpu::DeviceDescriptor) -> PixelsOptions {
        self.device_descriptor = dd;
        self
    }

    /// Create a frame buffer from the options builder.
    ///
    /// # Errors
    ///
    /// Returns an error when a `wgpu::Adapter` cannot be found or shaders
    /// are invalid SPIR-V.
    pub fn build(self, width: u32, height: u32, surface: &wgpu::Surface) -> Result<Pixels, Error> {
        Pixels::new_with_options(width, height, surface, self)
    }
}

impl Default for PixelsOptions {
    fn default() -> PixelsOptions {
        PixelsOptions {
            request_adapter_options: wgpu::RequestAdapterOptions::default(),
            device_descriptor: wgpu::DeviceDescriptor::default(),
        }
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
