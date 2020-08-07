//! A tiny library providing a GPU-powered pixel buffer.
//!
//! [`Pixels`] represents a 2D pixel buffer with an explicit image resolution, making it ideal for
//! prototyping simple pixel-based games, animations, and emulators. The pixel buffer is rendered
//! entirely on the GPU, allowing developers to easily incorporate special effects with shaders and
//! a customizable pipeline.
//!
//! The GPU interface is offered by [`wgpu`](https://crates.io/crates/wgpu), and is re-exported for
//! your convenience. Use a windowing framework or context manager of your choice;
//! [`winit`](https://crates.io/crates/winit) is a good place to start.
//!
//! # Environment variables
//!
//! * `PIXELS_HIGH_PERF`: Switch the default adapter to high performance.
//! * `PIXELS_LOW_POWER`: Switch the default adapter to low power.
//!
//! These variables change the default adapter to request either high performance or low power.
//! (I.e. discrete or integrated GPUs.) The value is not checked, only the existence
//! of the variable is relevant.
//!
//! The order of precedence for choosing a power preference is:
//!
//! 1. Application's specific adapter request through [`PixelsBuilder::request_adapter_options`]
//! 2. `PIXELS_HIGH_PERF`
//! 3. `PIXELS_LOW_POWER`
//! 4. `wgpu` default power preference (usually low power)

#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::env;

pub use crate::macros::*;
pub use crate::renderers::ScalingRenderer;
use thiserror::Error;
pub use wgpu;

mod macros;
mod renderers;

/// A logical texture for a window surface.
#[derive(Debug)]
pub struct SurfaceTexture {
    surface: wgpu::Surface,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub struct PixelsContext {
    // WGPU state
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,

    // Texture state for the source
    pub texture: wgpu::Texture,
    pub texture_extent: wgpu::Extent3d,
    pub texture_format_size: u32,

    // A default renderer to scale the input texture to the screen size
    pub scaling_renderer: ScalingRenderer,
}

/// Represents a 2D pixel buffer with an explicit image resolution.
///
/// See [`PixelsBuilder`] for building a customized pixel buffer.
#[derive(Debug)]
pub struct Pixels {
    context: PixelsContext,
    surface_texture: SurfaceTexture,
    present_mode: wgpu::PresentMode,

    // Pixel buffer
    pixels: Vec<u8>,

    // The inverse of the scaling matrix used by the renderer
    // Used to convert physical coordinates back to pixel coordinates (for the mouse)
    scaling_matrix_inverse: ultraviolet::Mat4,
}

/// A builder to help create customized pixel buffers.
pub struct PixelsBuilder<'req> {
    request_adapter_options: Option<wgpu::RequestAdapterOptions<'req>>,
    device_descriptor: wgpu::DeviceDescriptor,
    backend: wgpu::BackendBit,
    width: u32,
    height: u32,
    pixel_aspect_ratio: f64,
    present_mode: wgpu::PresentMode,
    surface_texture: SurfaceTexture,
    texture_format: wgpu::TextureFormat,
}

/// All the ways in which creating a pixel buffer can fail.
#[derive(Error, Debug)]
pub enum Error {
    /// No suitable [`wgpu::Adapter`] found
    #[error("No suitable `wgpu::Adapter` found")]
    AdapterNotFound,
    /// Equivalent to [`wgpu::TimeOut`]
    #[error("The GPU timed out when attempting to acquire the next texture or if a previous output is still alive.")]
    Timeout,
}

impl SurfaceTexture {
    /// Create a logical texture for a window surface.
    ///
    /// It is recommended (but not required) that the `width` and `height` are equivalent to the
    /// physical dimensions of the `surface`. E.g. scaled by the HiDPI factor.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pixels::{wgpu::Surface, SurfaceTexture};
    /// use winit::event_loop::EventLoop;
    /// use winit::window::Window;
    ///
    /// let event_loop = EventLoop::new();
    /// let window = Window::new(&event_loop).unwrap();
    /// let surface = Surface::create(&window);
    /// let size = window.inner_size();
    ///
    /// let width = size.width;
    /// let height = size.height;
    ///
    /// let surface_texture = SurfaceTexture::new(width, height, surface);
    /// # Ok::<(), pixels::Error>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when `width` or `height` are 0.
    pub fn new(width: u32, height: u32, surface: wgpu::Surface) -> SurfaceTexture {
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
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
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

    /// Resize the surface upon which the pixel buffer is rendered.
    ///
    /// This does not resize the pixel buffer. The pixel buffer will be fit onto the surface as
    /// best as possible by scaling to the nearest integer, e.g. 2x, 3x, 4x, etc.
    ///
    /// Call this method in response to a resize event from your window manager. The size expected
    /// is in physical pixel units.
    pub fn resize(&mut self, width: u32, height: u32) {
        // Update SurfaceTexture dimensions
        self.surface_texture.width = width;
        self.surface_texture.height = height;

        // Update ScalingMatrix for mouse transformation
        self.scaling_matrix_inverse = renderers::ScalingMatrix::new(
            (
                self.context.texture_extent.width as f32,
                self.context.texture_extent.height as f32,
            ),
            (width as f32, height as f32),
        )
        .transform
        .inversed();

        // Recreate the swap chain
        self.context.swap_chain = self.context.device.create_swap_chain(
            &self.surface_texture.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: self.surface_texture.width,
                height: self.surface_texture.height,
                present_mode: self.present_mode,
            },
        );

        // Update state for all render passes
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.context
            .scaling_renderer
            .resize(&mut self.context.device, &mut encoder, width, height);
        self.context.queue.submit(&[encoder.finish()]);
    }

    /// Draw this pixel buffer to the configured [`SurfaceTexture`].
    ///
    /// # Errors
    ///
    /// Returns an error when [`wgpu::SwapChain::get_next_texture`] times out.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    ///
    /// // Clear the pixel buffer
    /// let frame = pixels.get_frame();
    /// for pixel in frame.chunks_exact_mut(4) {
    ///     pixel[0] = 0x00; // R
    ///     pixel[1] = 0x00; // G
    ///     pixel[2] = 0x00; // B
    ///     pixel[3] = 0xff; // A
    /// }
    ///
    /// // Draw it to the `SurfaceTexture`
    /// pixels.render();
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn render(&mut self) -> Result<(), Error> {
        self.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);
        })
    }

    /// Draw this pixel buffer to the configured [`SurfaceTexture`] using a custom user-provided
    /// render function.
    ///
    /// Provides access to a [`wgpu::CommandEncoder`], a [`wgpu::TextureView`] from the swapchain
    /// which you can use to render to the screen, and a [`PixelsContext`] with all of the internal
    /// `wgpu` context.
    ///
    /// # Errors
    ///
    /// Returns an error when [`wgpu::SwapChain::get_next_texture`] times out.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    ///
    /// // Clear the pixel buffer
    /// let frame = pixels.get_frame();
    /// for pixel in frame.chunks_exact_mut(4) {
    ///     pixel[0] = 0x00; // R
    ///     pixel[1] = 0x00; // G
    ///     pixel[2] = 0x00; // B
    ///     pixel[3] = 0xff; // A
    /// }
    ///
    /// // Draw it to the `SurfaceTexture`
    /// pixels.render_with(|encoder, render_target, context| {
    ///     context.scaling_renderer.render(encoder, render_target);
    ///     // etc...
    /// });
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn render_with<F>(&mut self, render_function: F) -> Result<(), Error>
    where
        F: FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, &PixelsContext),
    {
        // TODO: Center frame buffer in surface
        let frame = self
            .context
            .swap_chain
            .get_next_texture()
            .map_err(|_| Error::Timeout)?;
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Update the pixel buffer texture view
        let mapped = self
            .context
            .device
            .create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: self.pixels.len() as u64,
                usage: wgpu::BufferUsage::COPY_SRC,
            });
        mapped.data.copy_from_slice(&self.pixels);
        let buffer = mapped.finish();

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: self.context.texture_extent.width * self.context.texture_format_size,
                rows_per_image: self.context.texture_extent.height,
            },
            wgpu::TextureCopyView {
                texture: &self.context.texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            self.context.texture_extent,
        );

        // Call the users render function.
        (render_function)(&mut encoder, &frame.view, &self.context);

        self.context.queue.submit(&[encoder.finish()]);
        Ok(())
    }

    /// Get a mutable byte slice for the pixel buffer. The buffer is _not_ cleared for you; it will
    /// retain the previous frame's contents until you clear it yourself.
    pub fn get_frame(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Calculate the pixel location from a physical location on the window,
    /// dealing with window resizing, scaling, and margins. Takes a physical
    /// position (x, y) within the window, and returns a pixel position (x, y).
    ///
    /// The location must be given in physical units (for example, winit's `PhysicalLocation`)
    ///
    /// If the given physical position is outside of the drawing area, this
    /// function returns an `Err` value with the pixel coordinates outside of
    /// the screen, using isize instead of usize.
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// const WIDTH:  u32 = 320;
    /// const HEIGHT: u32 = 240;
    ///
    /// let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
    ///
    /// // A cursor position in physical units
    /// let cursor_position: (f32, f32) = winit::dpi::PhysicalPosition::new(0.0, 0.0).into();
    ///
    /// // Convert it to a pixel location
    /// let pixel_position: (usize, usize) = pixels.window_pos_to_pixel(cursor_position)
    ///     // Clamp the output to within the screen
    ///     .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn window_pos_to_pixel(
        &self,
        physical_position: (f32, f32),
    ) -> Result<(usize, usize), (isize, isize)> {
        let physical_width = self.surface_texture.width as f32;
        let physical_height = self.surface_texture.height as f32;

        let pixels_width = self.context.texture_extent.width as f32;
        let pixels_height = self.context.texture_extent.height as f32;

        let pos = ultraviolet::Vec4::new(
            (physical_position.0 / physical_width - 0.5) * pixels_width,
            (physical_position.1 / physical_height - 0.5) * pixels_height,
            0.0,
            1.0,
        );

        let pos = self.scaling_matrix_inverse * pos;

        let pos = (
            pos.x / pos.w + pixels_width / 2.0,
            -pos.y / pos.w + pixels_height / 2.0,
        );
        let pixel_x = pos.0.floor() as isize;
        let pixel_y = pos.1.floor() as isize;

        if pixel_x < 0
            || pixel_x >= self.context.texture_extent.width as isize
            || pixel_y < 0
            || pixel_y >= self.context.texture_extent.height as isize
        {
            Err((pixel_x, pixel_y))
        } else {
            Ok((pixel_x as usize, pixel_y as usize))
        }
    }

    /// Clamp a pixel position to the pixel buffer size.
    ///
    /// This can be used to clamp the `Err` value returned by [`Pixels::window_pos_to_pixel`]
    /// to a position clamped within the drawing area.
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// const WIDTH:  u32 = 320;
    /// const HEIGHT: u32 = 240;
    ///
    /// let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
    ///
    /// let pixel_pos = pixels.clamp_pixel_pos((-19, 20));
    /// assert_eq!(pixel_pos, (0, 20));
    ///
    /// let pixel_pos = pixels.clamp_pixel_pos((11, 3000));
    /// assert_eq!(pixel_pos, (11, 239));
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn clamp_pixel_pos(&self, pos: (isize, isize)) -> (usize, usize) {
        (
            pos.0
                .max(0)
                .min(self.context.texture_extent.width as isize - 1) as usize,
            pos.1
                .max(0)
                .min(self.context.texture_extent.height as isize - 1) as usize,
        )
    }

    /// Provides access to the internal [`PixelsContext`]
    pub fn context(&mut self) -> &mut PixelsContext {
        &mut self.context
    }
}

impl<'req> PixelsBuilder<'req> {
    /// Create a builder that can be finalized into a [`Pixels`] pixel buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pixels::PixelsBuilder;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// let mut pixels = PixelsBuilder::new(256, 240, surface_texture)
    ///     .request_adapter_options(wgpu::RequestAdapterOptions {
    ///         power_preference: wgpu::PowerPreference::HighPerformance,
    ///         compatible_surface: None,
    ///     })
    ///     .enable_vsync(false)
    ///     .build()?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when `width` or `height` are 0.
    pub fn new(width: u32, height: u32, surface_texture: SurfaceTexture) -> PixelsBuilder<'req> {
        assert!(width > 0);
        assert!(height > 0);

        PixelsBuilder {
            request_adapter_options: None,
            device_descriptor: wgpu::DeviceDescriptor::default(),
            backend: wgpu::BackendBit::PRIMARY,
            width,
            height,
            pixel_aspect_ratio: 1.0,
            present_mode: wgpu::PresentMode::Fifo,
            surface_texture,
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        }
    }

    /// Add options for requesting a [`wgpu::Adapter`].
    pub const fn request_adapter_options(
        mut self,
        request_adapter_options: wgpu::RequestAdapterOptions<'req>,
    ) -> PixelsBuilder {
        self.request_adapter_options = Some(request_adapter_options);
        self
    }

    /// Add options for requesting a [`wgpu::Device`].
    pub const fn device_descriptor(
        mut self,
        device_descriptor: wgpu::DeviceDescriptor,
    ) -> PixelsBuilder<'req> {
        self.device_descriptor = device_descriptor;
        self
    }

    /// Set which backends wgpu will attempt to use.
    ///
    /// The default value of this is [`wgpu::BackendBit::PRIMARY`], which enables
    /// the well supported backends for wgpu.
    pub const fn wgpu_backend(mut self, backend: wgpu::BackendBit) -> PixelsBuilder<'req> {
        self.backend = backend;
        self
    }

    /// Set the pixel aspect ratio to simulate non-square pixels.
    ///
    /// This setting enables a render pass that horizontally scales the pixel buffer by the given
    /// factor.
    ///
    /// E.g. set this to `8.0 / 7.0` for an 8:7 pixel aspect ratio.
    ///
    /// # Panics
    ///
    /// The aspect ratio must be > 0.
    ///
    /// # Warning
    ///
    /// This documentation is hidden because support for pixel aspect ratio is incomplete.
    #[doc(hidden)]
    pub fn pixel_aspect_ratio(mut self, pixel_aspect_ratio: f64) -> PixelsBuilder<'req> {
        assert!(pixel_aspect_ratio > 0.0);

        self.pixel_aspect_ratio = pixel_aspect_ratio;
        self
    }

    /// Enable or disable Vsync.
    ///
    /// Vsync is enabled by default.
    ///
    /// The `wgpu` present mode will be set to `Fifo` when Vsync is enabled, or `Immediate` when
    /// Vsync is disabled. To set the present mode to `Mailbox` or another value, use the
    /// [`PixelsBuilder::present_mode`] method.
    pub fn enable_vsync(mut self, enable_vsync: bool) -> PixelsBuilder<'req> {
        self.present_mode = if enable_vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Immediate
        };
        self
    }

    /// Set the `wgpu` present mode.
    ///
    /// This differs from [`PixelsBuilder::enable_vsync`] by allowing the present mode to be set to
    /// any value.
    pub fn present_mode(mut self, present_mode: wgpu::PresentMode) -> PixelsBuilder<'req> {
        self.present_mode = present_mode;
        self
    }

    /// Set the texture format.
    ///
    /// The default value is [`wgpu::TextureFormat::Rgba8UnormSrgb`], which is 4 unsigned bytes in
    /// `RGBA` order using the SRGB color space. This is typically what you want when you are
    /// working with color values from popular image editing tools or web apps.
    pub const fn texture_format(
        mut self,
        texture_format: wgpu::TextureFormat,
    ) -> PixelsBuilder<'req> {
        self.texture_format = texture_format;
        self
    }

    /// Create a pixel buffer from the options builder.
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] cannot be found.
    pub fn build(self) -> Result<Pixels, Error> {
        // TODO: Use `options.pixel_aspect_ratio` to stretch the scaled texture
        let compatible_surface = Some(&self.surface_texture.surface);
        let adapter = pollster::block_on(wgpu::Adapter::request(
            &self.request_adapter_options.map_or_else(
                || wgpu::RequestAdapterOptions {
                    compatible_surface,
                    power_preference: get_default_power_preference(),
                },
                |rao| wgpu::RequestAdapterOptions {
                    compatible_surface: rao.compatible_surface.or(compatible_surface),
                    power_preference: rao.power_preference,
                },
            ),
            self.backend,
        ))
        .ok_or(Error::AdapterNotFound)?;

        let (mut device, queue) =
            pollster::block_on(adapter.request_device(&self.device_descriptor));

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
            label: None,
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

        // Create the pixel buffer
        let capacity = (width * height * texture_format_size) as usize;
        let mut pixels = Vec::with_capacity(capacity);
        pixels.resize_with(capacity, Default::default);

        let present_mode = self.present_mode;

        // Create swap chain
        let surface_texture = self.surface_texture;
        let swap_chain = device.create_swap_chain(
            &surface_texture.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: surface_texture.width,
                height: surface_texture.height,
                present_mode,
            },
        );

        let scaling_matrix_inverse = renderers::ScalingMatrix::new(
            (width as f32, height as f32),
            (surface_texture.width as f32, surface_texture.height as f32),
        )
        .transform
        .inversed();

        let scaling_renderer = ScalingRenderer::new(&device, &texture_view, &texture_extent);

        let context = PixelsContext {
            device,
            queue,
            swap_chain,
            texture,
            texture_extent,
            texture_format_size,
            scaling_renderer,
        };

        Ok(Pixels {
            context,
            surface_texture,
            present_mode,
            pixels,
            scaling_matrix_inverse,
        })
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
        wgpu::TextureFormat::R16Uint
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
        | wgpu::TextureFormat::Rgba16Uint
        | wgpu::TextureFormat::Rgba16Sint
        | wgpu::TextureFormat::Rgba16Float => 8,

        // 128-bit formats
        wgpu::TextureFormat::Rgba32Uint
        | wgpu::TextureFormat::Rgba32Sint
        | wgpu::TextureFormat::Rgba32Float => 16,
    }
}

fn get_default_power_preference() -> wgpu::PowerPreference {
    env::var("PIXELS_HIGH_PERF").map_or_else(
        |_| {
            env::var("PIXELS_LOW_POWER").map_or(wgpu::PowerPreference::Default, |_| {
                wgpu::PowerPreference::LowPower
            })
        },
        |_| wgpu::PowerPreference::HighPerformance,
    )
}
