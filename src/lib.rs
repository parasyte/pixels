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

#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

pub use crate::macros::*;
pub use crate::render_pass::{BoxedRenderPass, Device, Queue, RenderPass};
use crate::renderers::Renderer;
use thiserror::Error;
pub use wgpu;
use wgpu::{Extent3d, TextureView};

mod macros;
mod render_pass;
mod renderers;

type RenderPassFactory = Box<dyn Fn(Device, Queue, &TextureView, &Extent3d) -> BoxedRenderPass>;

/// A logical texture for a window surface.
#[derive(Debug)]
pub struct SurfaceTexture {
    surface: wgpu::Surface,
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
    surface_texture: SurfaceTexture,

    // List of render passes
    renderers: Vec<BoxedRenderPass>,

    // Texture state for the texel upload
    texture: wgpu::Texture,
    texture_extent: wgpu::Extent3d,
    texture_format_size: u32,
    pixels: Vec<u8>,
}

/// A builder to help create customized pixel buffers.
pub struct PixelsBuilder<'req> {
    request_adapter_options: wgpu::RequestAdapterOptions<'req>,
    device_descriptor: wgpu::DeviceDescriptor,
    width: u32,
    height: u32,
    pixel_aspect_ratio: f64,
    surface_texture: SurfaceTexture,
    texture_format: wgpu::TextureFormat,
    renderer_factories: Vec<RenderPassFactory>,
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
        // TODO: Call `update_bindings` on each render pass to create a texture chain

        // Update SurfaceTexture dimensions
        self.surface_texture.width = width;
        self.surface_texture.height = height;

        // Recreate the swap chain
        self.swap_chain = self.device.create_swap_chain(
            &self.surface_texture.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: self.surface_texture.width,
                height: self.surface_texture.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );

        // Update state for all render passes
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        for renderer in self.renderers.iter_mut() {
            renderer.resize(&mut encoder, width, height);
        }

        self.queue.borrow_mut().submit(&[encoder.finish()]);
    }

    /// Draw this pixel buffer to the configured [`SurfaceTexture`].
    ///
    /// This executes all render passes in sequence. See [`RenderPass`].
    ///
    /// # Errors
    ///
    /// Returns an error when [`wgpu::SwapChain::get_next_texture`] times out.
    pub fn render(&mut self) -> Result<(), Error> {
        // TODO: Center frame buffer in surface
        let frame = self
            .swap_chain
            .get_next_texture()
            .map_err(|_| Error::Timeout)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Update the pixel buffer texture view
        let mapped = self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
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
                bytes_per_row: self.texture_extent.width * self.texture_format_size,
                rows_per_image: self.texture_extent.height,
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            self.texture_extent,
        );

        // Execute all render passes
        for renderer in self.renderers.iter() {
            // TODO: Create a texture chain so that each pass receives the texture drawn by the previous
            renderer.render(&mut encoder, &frame.view);
        }

        self.queue.borrow_mut().submit(&[encoder.finish()]);
        Ok(())
    }

    /// Get a mutable byte slice for the pixel buffer. The buffer is _not_ cleared for you; it will
    /// retain the previous frame's contents until you clear it yourself.
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
    pub fn get_frame(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Calculate the pixel location from a physical location on the window,
    /// dealing with window resizing, scaling, and margins. Takes a physical
    /// position (x, y) within the window, and returns a pixel position (x, y).
    ///
    /// The location must be given in physical units (for example, winit's `PhysicalLocation`)
    ///
    /// If the given physical position is outside of the drawing area, the
    /// closest pixel is given by clamping the output in each dimension to the
    /// range from zero to the width or height of the surface minus one.
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    ///
    /// // A cursor position in physical units
    /// let cursor_position: (f64, f64) = winit::dpi::PhysicalPosition::new(0, 0).into();
    ///
    /// // Convert it to a pixel location
    /// let pixel_position: (usize, usize) = pixels.window_pos_to_pixel(cursor_position);
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn window_pos_to_pixel(&self, physical_position: (f64, f64)) -> (usize, usize) {
        let physical_size = (
            self.surface_texture.width as f64,
            self.surface_texture.height as f64,
        );

        let pixels_width = self.texture_extent.width as f64;
        let pixels_height = self.texture_extent.height as f64;

        // The scale factor used in the renderer
        let scale = (physical_size.0 / pixels_width)
            .min(physical_size.1 / pixels_height)
            .max(1.0)
            .floor();

        // Calculate the physical resolution of pixel's rendering area
        // (excluding the margins)
        let scaled_width = pixels_width * scale;
        let scaled_height = pixels_height * scale;

        // Calculate the widths of the margins
        let offset_x = (physical_size.0 as f64 - scaled_width) / 2.0;
        let offset_y = (physical_size.1 as f64 - scaled_height) / 2.0;

        // Calculate the pixel position of the two physical cursor positions
        let pixel_x = (physical_position.0 - offset_x).max(0.0).min(scaled_width) / scale;
        let pixel_y = (physical_position.1 - offset_y).max(0.0).min(scaled_height) / scale;

        (
            pixel_x.floor().min(pixels_width - 1.0) as usize,
            pixel_y.floor().min(pixels_height - 1.0) as usize,
        )
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
    /// struct MyRenderPass {
    ///     // ...
    /// };
    ///
    /// impl pixels::RenderPass for MyRenderPass {
    ///     // ...
    /// # fn update_bindings(&mut self, _: &wgpu::TextureView, _: &wgpu::Extent3d) {}
    /// # fn render(&self, _: &mut wgpu::CommandEncoder, _: &wgpu::TextureView) {}
    /// }
    ///
    /// let mut pixels = PixelsBuilder::new(256, 240, surface_texture)
    ///     .pixel_aspect_ratio(8.0 / 7.0)
    ///     .add_render_pass(|device, queue, texture, texture_size| {
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
    pub fn new(width: u32, height: u32, surface_texture: SurfaceTexture) -> PixelsBuilder<'req> {
        assert!(width > 0);
        assert!(height > 0);

        PixelsBuilder {
            request_adapter_options: wgpu::RequestAdapterOptions {
                compatible_surface: None,
                power_preference: wgpu::PowerPreference::Default,
            },
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
    pub const fn request_adapter_options(
        mut self,
        request_adapter_options: wgpu::RequestAdapterOptions<'req>,
    ) -> PixelsBuilder {
        self.request_adapter_options = request_adapter_options;
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
    pub fn pixel_aspect_ratio(mut self, pixel_aspect_ratio: f64) -> PixelsBuilder<'req> {
        assert!(pixel_aspect_ratio > 0.0);

        self.pixel_aspect_ratio = pixel_aspect_ratio;
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

    /// Add a render pass.
    ///
    /// Render passes are executed in the order they are added.
    ///
    /// # Factory Arguments
    ///
    /// * `device` - A reference-counted [`wgpu::Device`] which allows you to create GPU resources.
    /// * `queue` - A reference-counted [`wgpu::Queue`] which can execute command buffers.
    /// * `texture` - A [`wgpu::TextureView`] reference that is used as the texture input for the
    ///   render pass.
    /// * `texture_size` - A [`wgpu::Extent3d`] providing the input texture size.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pixels::{BoxedRenderPass, Device, PixelsBuilder, Queue, RenderPass};
    /// use pixels::wgpu::{Extent3d, TextureView};
    ///
    /// struct MyRenderPass {
    ///     device: Device,
    ///     queue: Queue,
    /// }
    ///
    /// impl MyRenderPass {
    ///     fn factory(
    ///         device: Device,
    ///         queue: Queue,
    ///         texture: &TextureView,
    ///         texture_size: &Extent3d,
    ///     ) -> BoxedRenderPass {
    ///         // Create a bind group, pipeline, etc. and store all of the necessary state...
    ///         Box::new(MyRenderPass { device, queue })
    ///     }
    /// }
    ///
    /// impl RenderPass for MyRenderPass {
    ///     // ...
    /// # fn update_bindings(&mut self, _: &wgpu::TextureView, _: &wgpu::Extent3d) {}
    /// # fn render(&self, _: &mut wgpu::CommandEncoder, _: &wgpu::TextureView) {}
    /// }
    ///
    /// # let surface = wgpu::Surface::create(&pixels_mocks::RWH);
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, surface);
    /// let builder = PixelsBuilder::new(320, 240, surface_texture)
    ///     .add_render_pass(MyRenderPass::factory)
    ///     .build()?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn add_render_pass(
        mut self,
        factory: impl Fn(Device, Queue, &TextureView, &Extent3d) -> BoxedRenderPass + 'static,
    ) -> PixelsBuilder<'req> {
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
        let adapter = futures_executor::block_on(wgpu::Adapter::request(
            &self.request_adapter_options,
            wgpu::BackendBit::PRIMARY,
        ))
        .ok_or(Error::AdapterNotFound)?;
        let (device, queue) =
            futures_executor::block_on(adapter.request_device(&self.device_descriptor));
        let device = Rc::new(device);
        let queue = Rc::new(RefCell::new(queue));

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

        // Create swap chain
        let surface_texture = self.surface_texture;
        let swap_chain = device.create_swap_chain(
            &surface_texture.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: surface_texture.width,
                height: surface_texture.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );

        // Create a renderer that impls `RenderPass`
        let mut renderers = vec![Renderer::factory(
            device.clone(),
            queue.clone(),
            &texture_view,
            &texture_extent,
        )];

        // Create all render passes
        renderers.extend(self.renderer_factories.iter().map(|f| {
            // TODO: Create a texture chain so that each pass receives the texture drawn by the previous
            f(
                device.clone(),
                queue.clone(),
                &texture_view,
                &texture_extent,
            )
        }));

        Ok(Pixels {
            device,
            queue,
            swap_chain,
            surface_texture,
            renderers,
            texture,
            texture_extent,
            texture_format_size,
            pixels,
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
