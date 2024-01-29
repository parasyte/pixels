//! A tiny library providing a GPU-powered pixel buffer.
//!
//! [`Pixels`] represents a 2D pixel buffer with an explicit image resolution, making it ideal for
//! prototyping simple pixel-based games, animations, and emulators. The pixel buffer is rendered
//! entirely on the GPU, allowing developers to easily incorporate special effects with shaders and
//! a customizable pipeline.
//!
//! The GPU interface is offered by [`wgpu`](https://crates.io/crates/wgpu), and is re-exported for
//! your convenience. Use a windowing framework or context manager of your choice;
//! [`winit`](https://crates.io/crates/winit) is a good place to start. Any windowing framework that
//! uses [`raw-window-handle`](https://crates.io/crates/raw-window-handle) will work.
//!
//! # Environment variables
//!
//! Pixels will default to selecting the most powerful GPU and most modern graphics API available on
//! the system, and these choices can be overridden with environment variables. These are the same
//! vars supported by the [`wgpu` examples](https://github.com/gfx-rs/wgpu/tree/v0.10/wgpu#usage).
//!
//! * `WGPU_BACKEND`: Select the backend (aka graphics API).
//!     * Supported values: `vulkan`, `metal`, `dx11`, `dx12`, `gl`, `webgpu`
//!     * The default depends on capabilities of the host system, with `vulkan` being preferred on
//!       Linux and Windows, and `metal` preferred on macOS.
//! * `WGPU_ADAPTER_NAME`: Select an adapter (aka GPU) with substring matching.
//!     * E.g. `1080` will match `NVIDIA GeForce 1080ti`
//! * `WGPU_POWER_PREF`: Select an adapter (aka GPU) that meets the given power profile.
//!     * Supported values: `low`, `high`
//!     * The default is `low`. I.e. an integrated GPU will be preferred over a discrete GPU.
//!
//! Note that `WGPU_ADAPTER_NAME` and `WGPU_POWER_PREF` are mutually exclusive and that
//! `WGPU_ADAPTER_NAME` takes precedence.

#![deny(clippy::all)]

pub use crate::builder::{check_texture_size, PixelsBuilder};
pub use crate::renderers::ScalingRenderer;
pub use raw_window_handle;
use thiserror::Error;
pub use wgpu;

mod builder;
mod renderers;

/// A logical texture for a window surface.
#[derive(Debug)]
pub struct SurfaceTexture<W: wgpu::WindowHandle> {
    window: W,
    size: SurfaceSize,
}

/// A logical texture size for a window surface.
#[derive(Debug)]
struct SurfaceSize {
    width: u32,
    height: u32,
}

/// Provides the internal state for custom shaders.
///
/// A reference to this struct is given to the `render_function` closure when using
/// [`Pixels::render_with`].
#[derive(Debug)]
pub struct PixelsContext<'win> {
    /// The `Device` allows creating GPU resources.
    pub device: wgpu::Device,

    /// The `Queue` provides access to the GPU command queue.
    pub queue: wgpu::Queue,

    surface: wgpu::Surface<'win>,

    /// This is the texture that your raw data is copied to by [`Pixels::render`] or
    /// [`Pixels::render_with`].
    pub texture: wgpu::Texture,

    /// Provides access to the texture size.
    pub texture_extent: wgpu::Extent3d,
    pub texture_format: wgpu::TextureFormat,

    /// Defines the "data rate" for the raw texture data. This is effectively the "bytes per pixel"
    /// count.
    ///
    /// Compressed textures may have less than one byte per pixel.
    pub texture_format_size: f32,

    /// A default renderer to scale the input texture to the screen size.
    pub scaling_renderer: ScalingRenderer,
}

/// Represents a 2D pixel buffer with an explicit image resolution.
///
/// See [`PixelsBuilder`] for building a customized pixel buffer.
#[derive(Debug)]
pub struct Pixels<'win> {
    context: PixelsContext<'win>,
    surface_size: SurfaceSize,
    present_mode: wgpu::PresentMode,
    render_texture_format: wgpu::TextureFormat,
    surface_texture_format: wgpu::TextureFormat,
    blend_state: wgpu::BlendState,
    alpha_mode: wgpu::CompositeAlphaMode,
    adapter: wgpu::Adapter,

    // Pixel buffer
    pixels: Vec<u8>,

    // The inverse of the scaling matrix used by the renderer
    // Used to convert physical coordinates back to pixel coordinates (for the mouse)
    scaling_matrix_inverse: ultraviolet::Mat4,
}

/// All the ways in which creating a pixel buffer can fail.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No suitable [`wgpu::Adapter`] found
    #[error("No suitable `wgpu::Adapter` found.")]
    AdapterNotFound,
    /// Equivalent to [`wgpu::RequestDeviceError`]
    #[error("No wgpu::Device found.")]
    DeviceNotFound(#[from] wgpu::RequestDeviceError),
    /// Equivalent to [`wgpu::SurfaceError`]
    #[error("The GPU failed to acquire a surface frame.")]
    Surface(#[from] wgpu::SurfaceError),
    /// Equivalent to [`wgpu::CreateSurfaceError`]
    #[error("Unable to create a surface.")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    /// Equivalent to [`TextureError`]
    #[error("Texture creation failed: {0}")]
    InvalidTexture(#[from] TextureError),
    /// User-defined error from custom render function
    #[error("User-defined error.")]
    UserDefined(#[from] DynError),
}

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// All the ways in which creating a texture can fail.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TextureError {
    /// Unable to create a backing texture; Width is either 0 or greater than GPU limits
    #[error("Texture width is invalid: {0}")]
    TextureWidth(u32),
    /// Unable to create a backing texture; Height is either 0 or greater than GPU limits
    #[error("Texture height is invalid: {0}")]
    TextureHeight(u32),
}

impl<W: wgpu::WindowHandle> SurfaceTexture<W> {
    /// Create a logical texture for a window surface.
    ///
    /// It is recommended (but not required) that the `width` and `height` are equivalent to the
    /// physical dimensions of the `surface`. E.g. scaled by the HiDPI factor.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pixels::SurfaceTexture;
    /// use winit::event_loop::EventLoop;
    /// use winit::window::Window;
    ///
    /// let event_loop = EventLoop::new().unwrap();
    /// let window = Window::new(&event_loop).unwrap();
    /// let size = window.inner_size();
    ///
    /// let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    /// # Ok::<(), pixels::Error>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when `width` or `height` are 0.
    pub fn new(width: u32, height: u32, window: W) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        let size = SurfaceSize { width, height };

        Self { window, size }
    }
}

impl<'win> Pixels<'win> {
    /// Create a pixel buffer instance with default options.
    ///
    /// Any ratio differences between the pixel buffer texture size and surface texture size will
    /// result in a border being added around the pixel buffer texture to maintain an integer
    /// scaling ratio.
    ///
    /// For instance, a pixel buffer with `320x240` can be scaled to a surface texture with sizes
    /// `320x240`, `640x480`, `960x720`, etc. without adding a border because these are exactly
    /// 1x, 2x, and 3x scales, respectively.
    ///
    /// This method blocks the current thread, making it unusable on Web targets. Use
    /// [`Pixels::new_async`] for a non-blocking alternative.
    ///
    /// # Examples
    ///
    /// Pass a borrowed window object to receive a `Pixels` object tied to the corresponding
    /// lifetime:
    ///
    /// ```no_run
    /// # use pixels::{Pixels, SurfaceTexture};
    /// # let window = pixels_mocks::Window;
    /// let surface_texture = SurfaceTexture::new(320, 240, &window);
    /// let mut pixels: Pixels<'_> = Pixels::new(320, 240, surface_texture)?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    ///
    /// Pass an owned window object to receive a static `Pixels` object, not tied to any lifetime.
    /// This includes objects wrapped in smart pointers like `Arc`, `Rc`, or `Box`:
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use pixels::{Pixels, SurfaceTexture};
    /// # let window = pixels_mocks::Window;
    /// let arc = Arc::new(window);
    /// let surface_texture = SurfaceTexture::new(320, 240, arc.clone());
    /// let mut pixels: Pixels<'static> = Pixels::new(320, 240, surface_texture)?;
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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<W: wgpu::WindowHandle + 'win>(
        width: u32,
        height: u32,
        surface_texture: SurfaceTexture<W>,
    ) -> Result<Self, Error> {
        PixelsBuilder::new(width, height, surface_texture).build()
    }

    /// Asynchronously create a pixel buffer instance with default options.
    ///
    /// See [`Pixels::new`] for more information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn test() -> Result<(), pixels::Error> {
    /// # use pixels::Pixels;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = Pixels::new_async(320, 240, surface_texture).await?;
    /// # Ok::<(), pixels::Error>(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] cannot be found.
    ///
    /// # Panics
    ///
    /// Panics when `width` or `height` are 0.
    pub async fn new_async<W: wgpu::WindowHandle + 'win>(
        width: u32,
        height: u32,
        surface_texture: SurfaceTexture<W>,
    ) -> Result<Self, Error> {
        PixelsBuilder::new(width, height, surface_texture)
            .build_async()
            .await
    }

    /// Change the clear color.
    ///
    /// Allows customization of the background color and the border drawn for non-integer scale
    /// values.
    ///
    /// ```no_run
    /// use pixels::wgpu::Color;
    ///
    /// # use pixels::Pixels;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    ///
    /// // Set clear color to red.
    /// pixels.clear_color(Color::RED);
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn clear_color(&mut self, color: wgpu::Color) {
        self.context.scaling_renderer.clear_color = color;
    }

    /// Returns a reference of the `wgpu` adapter used by the crate.
    ///
    /// The adapter can be used to retrieve runtime information about the host system
    /// or the WGPU backend.
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    /// let adapter = pixels.adapter();
    /// // Do something with the adapter.
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Resize the pixel buffer and zero its contents.
    ///
    /// This does not resize the surface upon which the pixel buffer texture is rendered. Use
    /// [`Pixels::resize_surface`] to change the size of the surface texture.
    ///
    /// The pixel buffer will be fit onto the surface texture as best as possible by scaling to the
    /// nearest integer, e.g. 2x, 3x, 4x, etc. A border will be added around the pixel buffer
    /// texture for non-integer scaling ratios.
    ///
    /// Call this method to change the virtual screen resolution. E.g. when you want your pixel
    /// buffer to be resized from `640x480` to `800x600`.
    ///
    /// # Errors
    ///
    /// - [`TextureError::TextureWidth`] when `width` is 0 or greater than GPU texture limits.
    /// - [`TextureError::TextureHeight`] when `height` is 0 or greater than GPU texture limits.
    pub fn resize_buffer(&mut self, width: u32, height: u32) -> Result<(), TextureError> {
        // Recreate the backing texture
        let (scaling_matrix_inverse, texture_extent, texture, scaling_renderer, pixels_buffer_size) =
            builder::create_backing_texture(
                &self.context.device,
                // Backing texture values
                width,
                height,
                self.context.texture_format,
                // Render texture values
                &self.surface_size,
                self.render_texture_format,
                self.context.scaling_renderer.clear_color,
                self.blend_state,
            )?;

        self.scaling_matrix_inverse = scaling_matrix_inverse;
        self.context.texture_extent = texture_extent;
        self.context.texture = texture;
        self.context.scaling_renderer = scaling_renderer;

        // Resize the pixel buffer
        self.pixels
            .resize_with(pixels_buffer_size, Default::default);

        Ok(())
    }

    /// Resize the surface upon which the pixel buffer texture is rendered.
    ///
    /// This does not resize the pixel buffer. Use [`Pixels::resize_buffer`] to change the size of
    /// the pixel buffer.
    ///
    /// The pixel buffer texture will be fit onto the surface texture as best as possible by scaling
    /// to the nearest integer, e.g. 2x, 3x, 4x, etc. A border will be added around the pixel buffer
    /// texture for non-integer scaling ratios.
    ///
    /// Call this method in response to a resize event from your window manager. The size expected
    /// is in physical pixel units. Does nothing when `width` or `height` are 0.
    ///
    /// # Errors
    ///
    /// - [`TextureError::TextureWidth`] when `width` is 0 or greater than GPU texture limits.
    /// - [`TextureError::TextureHeight`] when `height` is 0 or greater than GPU texture limits.
    pub fn resize_surface(&mut self, width: u32, height: u32) -> Result<(), TextureError> {
        check_texture_size(&self.context.device, width, height)?;

        // Update SurfaceTexture dimensions
        self.surface_size.width = width;
        self.surface_size.height = height;

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

        // Reconfigure the surface
        self.reconfigure_surface();

        // Update state for all render passes
        self.context
            .scaling_renderer
            .resize(&self.context.queue, width, height);

        Ok(())
    }

    /// Enable or disable Vsync.
    ///
    /// Vsync is enabled by default. It cannot be disabled on Web targets.
    ///
    /// The `wgpu` present mode will be set to `AutoVsync` when Vsync is enabled, or `AutoNoVsync`
    /// when Vsync is disabled. To set the present mode to `Mailbox` or another value, use the
    /// [`Pixels::set_present_mode`] method.
    pub fn enable_vsync(&mut self, enable_vsync: bool) {
        self.present_mode = if enable_vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.reconfigure_surface();
    }

    /// Get the `wgpu` present mode.
    ///
    /// Returns the present mode currently in use by the surface, which can be changed through
    /// [`Pixels::enable_vsync`] or [`Pixels::set_present_mode`].
    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.present_mode
    }

    /// Set the `wgpu` present mode.
    ///
    /// This differs from [`Pixels::enable_vsync`] by allowing the present mode to be set to
    /// any value.
    pub fn set_present_mode(&mut self, present_mode: wgpu::PresentMode) {
        self.present_mode = present_mode;
        self.reconfigure_surface();
    }

    /// Draw this pixel buffer to the configured [`SurfaceTexture`].
    ///
    /// # Errors
    ///
    /// Returns an error when [`wgpu::Surface::get_current_texture`] fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    ///
    /// // Clear the pixel buffer
    /// let frame = pixels.frame_mut();
    /// for pixel in frame.chunks_exact_mut(4) {
    ///     pixel[0] = 0x00; // R
    ///     pixel[1] = 0x00; // G
    ///     pixel[2] = 0x00; // B
    ///     pixel[3] = 0xff; // A
    /// }
    ///
    /// // Draw it to the `SurfaceTexture`
    /// pixels.render()?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn render(&self) -> Result<(), Error> {
        self.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);

            Ok(())
        })
    }

    /// Draw this pixel buffer to the configured [`SurfaceTexture`] using a custom user-provided
    /// render function.
    ///
    /// Provides access to a [`wgpu::CommandEncoder`], a [`wgpu::TextureView`] from the surface
    /// which you can use to render to the screen, and a [`PixelsContext`] with all of the internal
    /// `wgpu` context.
    ///
    /// The render function must return a `Result`. This allows fallible render functions to be
    /// handled gracefully. The boxed `Error` will be made available in the [`Error::UserDefined`]
    /// variant returned by `render_with()`.
    ///
    /// # Errors
    ///
    /// Returns an error when either [`wgpu::Surface::get_current_texture`] or the provided render
    /// function fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    ///
    /// // Clear the pixel buffer
    /// let frame = pixels.frame_mut();
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
    ///     Ok(())
    /// })?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn render_with<F>(&self, render_function: F) -> Result<(), Error>
    where
        F: FnOnce(
            &mut wgpu::CommandEncoder,
            &wgpu::TextureView,
            &PixelsContext,
        ) -> Result<(), DynError>,
    {
        let frame = self.context.surface.get_current_texture().or_else(|_| {
            // Reconfigure the surface and retry immediately on any error.
            // See https://github.com/parasyte/pixels/issues/121
            // See https://github.com/parasyte/pixels/issues/346
            self.reconfigure_surface();
            self.context.surface.get_current_texture()
        })?;
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("pixels_command_encoder"),
                });

        // Update the pixel buffer texture view
        let bytes_per_row =
            (self.context.texture_extent.width as f32 * self.context.texture_format_size) as u32;
        self.context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.context.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(self.context.texture_extent.height),
            },
            self.context.texture_extent,
        );

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Call the user's render function.
        (render_function)(&mut encoder, &view, &self.context)?;

        self.context.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    /// Reconfigure the surface.
    ///
    /// Call this when the surface or presentation mode needs to be changed.
    pub(crate) fn reconfigure_surface(&self) {
        self.context.surface.configure(
            &self.context.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_texture_format,
                width: self.surface_size.width,
                height: self.surface_size.height,
                present_mode: self.present_mode,
                desired_maximum_frame_latency: 2,
                alpha_mode: self.alpha_mode,
                view_formats: vec![],
            },
        );
    }

    /// Get a mutable byte slice for the pixel buffer. The buffer is _not_ cleared for you; it will
    /// retain the previous frame's contents until you clear it yourself.
    pub fn frame_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Get an immutable byte slice for the pixel buffer.
    ///
    /// This may be useful for operations that must sample the buffer, such as blending pixel
    /// colours directly into it.
    pub fn frame(&self) -> &[u8] {
        &self.pixels
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
    /// use winit::dpi::PhysicalPosition;
    ///
    /// # use pixels::Pixels;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
    ///
    /// // A cursor position in physical units
    /// let cursor_position: (f32, f32) = PhysicalPosition::new(0.0, 0.0).into();
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
        let physical_width = self.surface_size.width as f32;
        let physical_height = self.surface_size.height as f32;

        let pixels_width = self.context.texture_extent.width as f32;
        let pixels_height = self.context.texture_extent.height as f32;

        let pos = ultraviolet::Vec4::new(
            (physical_position.0 / physical_width - 0.5) * pixels_width,
            (physical_position.1 / physical_height - 0.5) * pixels_height,
            0.0,
            1.0,
        );

        let pos = self.scaling_matrix_inverse * pos;
        let offset_width = pixels_width.min(physical_width) / 2.0;
        let offset_height = pixels_height.min(physical_height) / 2.0;

        let pixel_x = (pos.x / pos.w + offset_width).floor() as isize;
        let pixel_y = (pos.y / pos.w + offset_height).floor() as isize;

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

    /// Clamp a pixel position to the pixel buffer texture size.
    ///
    /// This can be used to clamp the `Err` value returned by [`Pixels::window_pos_to_pixel`]
    /// to a position clamped within the drawing area.
    ///
    /// ```no_run
    /// # use pixels::Pixels;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = Pixels::new(320, 240, surface_texture)?;
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
                .clamp(0, self.context.texture_extent.width as isize - 1) as usize,
            pos.1
                .clamp(0, self.context.texture_extent.height as isize - 1) as usize,
        )
    }

    /// Provides access to the internal [`wgpu::Device`].
    pub fn device(&self) -> &wgpu::Device {
        &self.context.device
    }

    /// Provides access to the internal [`wgpu::Queue`].
    pub fn queue(&self) -> &wgpu::Queue {
        &self.context.queue
    }

    /// Provides access to the internal source [`wgpu::Texture`].
    ///
    /// This is the pre-scaled texture copied from the pixel buffer.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.context.texture
    }

    /// Provides access to the internal [`PixelsContext`].
    pub fn context(&self) -> &PixelsContext {
        &self.context
    }

    /// Get the surface texture format.
    ///
    /// This texture format may be chosen automatically by the surface. See
    /// [`PixelsBuilder::surface_texture_format`] for more information.
    pub fn surface_texture_format(&self) -> wgpu::TextureFormat {
        self.surface_texture_format
    }

    /// Get the render texture format.
    ///
    ///
    /// See [`PixelsBuilder::render_texture_format`] for more information.
    pub fn render_texture_format(&self) -> wgpu::TextureFormat {
        self.render_texture_format
    }
}
