use crate::renderers::{ScalingMatrix, ScalingRenderer};
use crate::{Error, Pixels, PixelsContext, SurfaceSize, SurfaceTexture, TextureError};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

/// A builder to help create customized pixel buffers.
pub struct PixelsBuilder<'req, 'dev, 'win, W: HasRawWindowHandle + HasRawDisplayHandle> {
    request_adapter_options: Option<wgpu::RequestAdapterOptions<'req>>,
    device_descriptor: Option<wgpu::DeviceDescriptor<'dev>>,
    backend: wgpu::Backends,
    width: u32,
    height: u32,
    _pixel_aspect_ratio: f64,
    present_mode: wgpu::PresentMode,
    surface_texture: SurfaceTexture<'win, W>,
    texture_format: wgpu::TextureFormat,
    render_texture_format: Option<wgpu::TextureFormat>,
    surface_texture_format: Option<wgpu::TextureFormat>,
    clear_color: wgpu::Color,
    blend_state: wgpu::BlendState,
}

impl<'req, 'dev, 'win, W: HasRawWindowHandle + HasRawDisplayHandle>
    PixelsBuilder<'req, 'dev, 'win, W>
{
    /// Create a builder that can be finalized into a [`Pixels`] pixel buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
    ///
    /// # use pixels::PixelsBuilder;
    /// # let window = pixels_mocks::Rwh;
    /// # let surface_texture = pixels::SurfaceTexture::new(256, 240, &window);
    /// let mut pixels = PixelsBuilder::new(256, 240, surface_texture)
    ///     .request_adapter_options(RequestAdapterOptions {
    ///         power_preference: PowerPreference::HighPerformance,
    ///         force_fallback_adapter: false,
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
    pub fn new(width: u32, height: u32, surface_texture: SurfaceTexture<'win, W>) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        Self {
            request_adapter_options: None,
            device_descriptor: None,
            backend: wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all),
            width,
            height,
            _pixel_aspect_ratio: 1.0,
            present_mode: wgpu::PresentMode::AutoVsync,
            surface_texture,
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            render_texture_format: None,
            surface_texture_format: None,
            clear_color: wgpu::Color::BLACK,
            blend_state: wgpu::BlendState::ALPHA_BLENDING,
        }
    }

    /// Add options for requesting a [`wgpu::Adapter`].
    pub fn request_adapter_options(
        mut self,
        request_adapter_options: wgpu::RequestAdapterOptions<'req>,
    ) -> Self {
        self.request_adapter_options = Some(request_adapter_options);
        self
    }

    /// Add options for requesting a [`wgpu::Device`].
    pub fn device_descriptor(mut self, device_descriptor: wgpu::DeviceDescriptor<'dev>) -> Self {
        self.device_descriptor = Some(device_descriptor);
        self
    }

    /// Set which backends wgpu will attempt to use.
    ///
    /// The default enables all backends, including the backends with "best effort" support in wgpu.
    pub fn wgpu_backend(mut self, backend: wgpu::Backends) -> Self {
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
    pub fn pixel_aspect_ratio(mut self, pixel_aspect_ratio: f64) -> Self {
        assert!(pixel_aspect_ratio > 0.0);

        self._pixel_aspect_ratio = pixel_aspect_ratio;
        self
    }

    /// Enable or disable Vsync.
    ///
    /// Vsync is enabled by default. It cannot be disabled on Web targets.
    ///
    /// The `wgpu` present mode will be set to `AutoVsync` when Vsync is enabled, or `AutoNoVsync`
    /// when Vsync is disabled. To set the present mode to `Mailbox` or another value, use the
    /// [`PixelsBuilder::present_mode`] method.
    pub fn enable_vsync(mut self, enable_vsync: bool) -> Self {
        self.present_mode = if enable_vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self
    }

    /// Set the `wgpu` present mode.
    ///
    /// This differs from [`PixelsBuilder::enable_vsync`] by allowing the present mode to be set to
    /// any value.
    pub fn present_mode(mut self, present_mode: wgpu::PresentMode) -> Self {
        self.present_mode = present_mode;
        self
    }

    /// Set the texture format.
    ///
    /// The default value is `Rgba8UnormSrgb`, which is 4 unsigned bytes in `RGBA` order using the
    /// sRGB color space. This is typically what you want when you are working with color values
    /// from popular image editing tools or web apps.
    ///
    /// This is the pixel format of the texture that most applications will interact with directly.
    /// The format influences the structure of byte data that is returned by [`Pixels::frame`].
    pub fn texture_format(mut self, texture_format: wgpu::TextureFormat) -> Self {
        self.texture_format = texture_format;
        self
    }

    /// Set the render texture format.
    ///
    /// This falls back on [`Pixels::surface_texture_format`] if not set.
    ///
    /// The [`ScalingRenderer`] uses this format for its own render target.
    /// This is really only useful if you are running a custom shader pipeline and need different formats
    /// for the intermediary textures (such as `Rgba16Float` for HDR rendering).
    /// There is a full example of a
    /// [custom-shader](https://github.com/parasyte/pixels/tree/master/examples/custom-shader)
    /// available that demonstrates how to deal with this.
    pub fn render_texture_format(mut self, texture_format: wgpu::TextureFormat) -> Self {
        self.render_texture_format = Some(texture_format);
        self
    }

    /// Set the surface texture format.
    ///
    /// The default value is chosen automatically by the surface (if it can) with a fallback to
    /// `Bgra8UnormSrgb` (which is 4 unsigned bytes in `BGRA` order using the sRGB color space).
    /// Setting this format correctly depends on the hardware/platform the pixel buffer is rendered
    /// to. The chosen format can be retrieved later with [`Pixels::render_texture_format`].
    ///
    /// This method controls the format of the surface frame buffer, which has strict texture
    /// format requirements. Applications will never interact directly with the pixel data of this
    /// texture, but a view is provided to the `render_function` closure by [`Pixels::render_with`].
    /// The render texture can only be used as the final render target at the end of all
    /// post-processing shaders.
    pub fn surface_texture_format(mut self, texture_format: wgpu::TextureFormat) -> Self {
        self.surface_texture_format = Some(texture_format);
        self
    }

    /// Set the blend state.
    ///
    /// Allows customization of how to mix the new and existing pixels in a texture
    /// when rendering.
    ///
    /// The default blend state is alpha blending with non-premultiplied alpha.
    ///
    /// ```no_run
    /// use pixels::wgpu::BlendState;
    ///
    /// # use pixels::PixelsBuilder;
    /// # let window = pixels_mocks::Rwh;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// // Replace the old pixels with the new without mixing.
    /// let mut pixels = PixelsBuilder::new(320, 240, surface_texture)
    ///     .blend_state(wgpu::BlendState::REPLACE)
    ///     .build()?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn blend_state(mut self, blend_state: wgpu::BlendState) -> Self {
        self.blend_state = blend_state;
        self
    }

    /// Set the clear color.
    ///
    /// Allows customization of the background color and the border drawn for non-integer scale
    /// values.
    ///
    /// The default value is pure black.
    ///
    /// ```no_run
    /// use pixels::wgpu::Color;
    ///
    /// # use pixels::PixelsBuilder;
    /// # let window = pixels_mocks::Rwh;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// // Set clear color to bright magenta.
    /// let mut pixels = PixelsBuilder::new(320, 240, surface_texture)
    ///     .clear_color(Color {
    ///         r: 1.0,
    ///         g: 0.0,
    ///         b: 1.0,
    ///         a: 1.0,
    ///     })
    ///     .build()?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn clear_color(mut self, color: wgpu::Color) -> Self {
        self.clear_color = color;
        self
    }

    /// Create a pixel buffer from the options builder.
    ///
    /// This is the private implementation shared by [`PixelsBuilder::build`] and
    /// [`PixelsBuilder::build_async`].
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] cannot be found.
    async fn build_impl(self) -> Result<Pixels, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: self.backend,
            ..Default::default()
        });

        // TODO: Use `options.pixel_aspect_ratio` to stretch the scaled texture
        let surface = unsafe { instance.create_surface(self.surface_texture.window) }?;
        let compatible_surface = Some(&surface);
        let request_adapter_options = &self.request_adapter_options;
        let adapter = match wgpu::util::initialize_adapter_from_env(&instance, compatible_surface) {
            Some(adapter) => Some(adapter),
            None => {
                instance
                    .request_adapter(&request_adapter_options.as_ref().map_or_else(
                        || wgpu::RequestAdapterOptions {
                            compatible_surface,
                            force_fallback_adapter: false,
                            power_preference:
                                wgpu::util::power_preference_from_env().unwrap_or_default(),
                        },
                        |rao| wgpu::RequestAdapterOptions {
                            compatible_surface: rao.compatible_surface.or(compatible_surface),
                            force_fallback_adapter: rao.force_fallback_adapter,
                            power_preference: rao.power_preference,
                        },
                    ))
                    .await
            }
        };

        let adapter = adapter.ok_or(Error::AdapterNotFound)?;

        let device_descriptor = self
            .device_descriptor
            .unwrap_or_else(|| wgpu::DeviceDescriptor {
                limits: adapter.limits(),
                ..wgpu::DeviceDescriptor::default()
            });

        let (device, queue) = adapter.request_device(&device_descriptor, None).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let present_mode = self.present_mode;
        let surface_texture_format = self.surface_texture_format.unwrap_or_else(|| {
            *surface_capabilities
                .formats
                .iter()
                .find(|format| format.is_srgb())
                .unwrap_or(&wgpu::TextureFormat::Bgra8UnormSrgb)
        });
        let render_texture_format = self.render_texture_format.unwrap_or(surface_texture_format);

        // Create the backing texture
        let surface_size = self.surface_texture.size;
        let clear_color = self.clear_color;
        let blend_state = self.blend_state;
        let (scaling_matrix_inverse, texture_extent, texture, scaling_renderer, pixels_buffer_size) =
            create_backing_texture(
                &device,
                // Backing texture values
                self.width,
                self.height,
                self.texture_format,
                // Render texture values
                &surface_size,
                render_texture_format,
                // Clear color and blending values
                clear_color,
                blend_state,
            )?;

        // Create the pixel buffer
        let mut pixels = Vec::with_capacity(pixels_buffer_size);
        pixels.resize_with(pixels_buffer_size, Default::default);

        let alpha_mode = surface_capabilities.alpha_modes[0];

        // Instantiate the Pixels struct
        let context = PixelsContext {
            device,
            queue,
            surface,
            texture,
            texture_extent,
            texture_format: self.texture_format,
            texture_format_size: texture_format_size(self.texture_format),
            scaling_renderer,
        };

        let pixels = Pixels {
            context,
            adapter,
            surface_size,
            present_mode,
            render_texture_format,
            surface_texture_format,
            blend_state,
            pixels,
            scaling_matrix_inverse,
            alpha_mode,
        };
        pixels.reconfigure_surface();

        Ok(pixels)
    }

    /// Create a pixel buffer from the options builder.
    ///
    /// This method blocks the current thread, making it unusable on Web targets. Use
    /// [`PixelsBuilder::build_async`] for a non-blocking alternative.
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] or [`wgpu::Device`] cannot be found.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn build(self) -> Result<Pixels, Error> {
        pollster::block_on(self.build_impl())
    }

    /// Create a pixel buffer from the options builder without blocking the current thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pixels::wgpu::{Backends, DeviceDescriptor, Limits};
    ///
    /// # async fn test() -> Result<(), pixels::Error> {
    /// # use pixels::PixelsBuilder;
    /// # let window = pixels_mocks::Rwh;
    /// # let surface_texture = pixels::SurfaceTexture::new(256, 240, &window);
    /// let mut pixels = PixelsBuilder::new(256, 240, surface_texture)
    ///     .enable_vsync(false)
    ///     .build_async()
    ///     .await?;
    /// # Ok::<(), pixels::Error>(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] or [`wgpu::Device`] cannot be found.
    pub async fn build_async(self) -> Result<Pixels, Error> {
        self.build_impl().await
    }
}

/// Compare the given size to the limits defined by `device`.
///
/// # Errors
///
/// - [`TextureError::TextureWidth`] when `width` is 0 or greater than GPU texture limits.
/// - [`TextureError::TextureHeight`] when `height` is 0 or greater than GPU texture limits.
pub fn check_texture_size(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> Result<(), TextureError> {
    let limits = device.limits();
    if width == 0 || width > limits.max_texture_dimension_2d {
        return Err(TextureError::TextureWidth(width));
    }
    if height == 0 || height > limits.max_texture_dimension_2d {
        return Err(TextureError::TextureHeight(height));
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_backing_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    backing_texture_format: wgpu::TextureFormat,
    surface_size: &SurfaceSize,
    render_texture_format: wgpu::TextureFormat,
    clear_color: wgpu::Color,
    blend_state: wgpu::BlendState,
) -> Result<
    (
        ultraviolet::Mat4,
        wgpu::Extent3d,
        wgpu::Texture,
        ScalingRenderer,
        usize,
    ),
    TextureError,
> {
    check_texture_size(device, width, height)?;

    let scaling_matrix_inverse = ScalingMatrix::new(
        (width as f32, height as f32),
        (surface_size.width as f32, surface_size.height as f32),
    )
    .transform
    .inversed();

    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pixels_source_texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: backing_texture_format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let scaling_renderer = ScalingRenderer::new(
        device,
        &texture_view,
        &texture_extent,
        surface_size,
        render_texture_format,
        clear_color,
        blend_state,
    );

    let texture_format_size = texture_format_size(backing_texture_format);
    let pixels_buffer_size = ((width * height) as f32 * texture_format_size) as usize;

    Ok((
        scaling_matrix_inverse,
        texture_extent,
        texture,
        scaling_renderer,
        pixels_buffer_size,
    ))
}

#[rustfmt::skip]
#[inline]
const fn texture_format_size(texture_format: wgpu::TextureFormat) -> f32 {
    use wgpu::{AstcBlock::*, TextureFormat::*};

    // TODO: Use constant arithmetic when supported.
    // See: https://github.com/rust-lang/rust/issues/57241
    match texture_format {
        // Note that these sizes are typically estimates. For instance, GPU vendors decide whether
        // their implementation uses 5 or 8 bytes per texel for formats like `Depth32PlusStencil8`.
        // In cases where it is unclear, we choose to overestimate.
        //
        // See:
        // - https://gpuweb.github.io/gpuweb/#plain-color-formats
        // - https://gpuweb.github.io/gpuweb/#depth-formats
        // - https://gpuweb.github.io/gpuweb/#packed-formats

        // 8-bit formats, 8 bits per component
        R8Unorm
        | R8Snorm
        | R8Uint
        | R8Sint
        | Stencil8 => 1.0, // 8.0 / 8.0

        // 16-bit formats, 8 bits per component
        R16Uint
        | R16Sint
        | R16Float
        | R16Unorm
        | R16Snorm
        | Rg8Unorm
        | Rg8Snorm
        | Rg8Uint
        | Rg8Sint
        | Rgb9e5Ufloat
        | Depth16Unorm => 2.0, // 16.0 / 8.0

        // 32-bit formats, 8 bits per component
        R32Uint
        | R32Sint
        | R32Float
        | Rg16Uint
        | Rg16Sint
        | Rg16Float
        | Rg16Unorm
        | Rg16Snorm
        | Rgba8Unorm
        | Rgba8UnormSrgb
        | Rgba8Snorm
        | Rgba8Uint
        | Rgba8Sint
        | Bgra8Unorm
        | Bgra8UnormSrgb
        | Rgb10a2Uint
        | Rgb10a2Unorm
        | Rg11b10Float
        | Depth32Float
        | Depth24Plus
        | Depth24PlusStencil8 => 4.0, // 32.0 / 8.0

        // 64-bit formats, 8 bits per component
        Rg32Uint
        | Rg32Sint
        | Rg32Float
        | Rgba16Uint
        | Rgba16Sint
        | Rgba16Float
        | Rgba16Unorm
        | Rgba16Snorm
        | Depth32FloatStencil8 => 8.0, // 64.0 / 8.0

        // 128-bit formats, 8 bits per component
        Rgba32Uint
        | Rgba32Sint
        | Rgba32Float => 16.0, // 128.0 / 8.0

        // Compressed formats

        // 4x4 blocks, 8 bytes per block
        Bc1RgbaUnorm
        | Bc1RgbaUnormSrgb
        | Bc4RUnorm
        | Bc4RSnorm
        | Etc2Rgb8Unorm
        | Etc2Rgb8UnormSrgb
        | Etc2Rgb8A1Unorm
        | Etc2Rgb8A1UnormSrgb
        | EacR11Unorm
        | EacR11Snorm => 0.5, // 4.0 * 4.0 / 8.0

        // 4x4 blocks, 16 bytes per block
        Bc2RgbaUnorm
        | Bc2RgbaUnormSrgb
        | Bc3RgbaUnorm
        | Bc3RgbaUnormSrgb
        | Bc5RgUnorm
        | Bc5RgSnorm
        | Bc6hRgbUfloat
        | Bc6hRgbFloat
        | Bc7RgbaUnorm
        | Bc7RgbaUnormSrgb
        | EacRg11Unorm
        | EacRg11Snorm
        | Etc2Rgba8Unorm
        | Etc2Rgba8UnormSrgb
        | Astc { block: B4x4, channel: _ } => 1.0, // 4.0 * 4.0 / 16.0

        // 5x4 blocks, 16 bytes per block
        Astc { block: B5x4, channel: _ } => 1.25, // 5.0 * 4.0 / 16.0

        // 5x5 blocks, 16 bytes per block
        Astc { block: B5x5, channel: _ } => 1.5625, // 5.0 * 5.0 / 16.0

        // 6x5 blocks, 16 bytes per block
        Astc { block: B6x5, channel: _ } => 1.875, // 6.0 * 5.0 / 16.0

        // 6x6 blocks, 16 bytes per block
        Astc { block: B6x6, channel: _ } => 2.25, // 6.0 * 6.0 / 16.0

        // 8x5 blocks, 16 bytes per block
        Astc { block: B8x5, channel: _ } => 2.5, // 8.0 * 5.0 / 16.0

        // 8x6 blocks, 16 bytes per block
        Astc { block: B8x6, channel: _ } => 3.0, // 8.0 * 6.0 / 16.0

        // 8x8 blocks, 16 bytes per block
        Astc { block: B8x8, channel: _ } => 4.0, // 8.0 * 8.0 / 16.0

        // 10x5 blocks, 16 bytes per block
        Astc { block: B10x5, channel: _ } => 3.125, // 10.0 * 5.0 / 16.0

        // 10x6 blocks, 16 bytes per block
        Astc { block: B10x6, channel: _ } => 3.75, // 10.0 * 6.0 / 16.0

        // 10x8 blocks, 16 bytes per block
        Astc { block: B10x8, channel: _ } => 5.0, // 10.0 * 8.0 / 16.0

        // 10x10 blocks, 16 bytes per block
        Astc { block: B10x10, channel: _ } => 6.25, // 10.0 * 10.0 / 16.0

        // 12x10 blocks, 16 bytes per block
        Astc { block: B12x10, channel: _ } => 7.5, // 12.0 * 10.0 / 16.0

        // 12x12 blocks, 16 bytes per block
        Astc { block: B12x12, channel: _ } => 9.0, // 12.0 * 12.0 / 16.0
    }
}
