use crate::renderers::{ScalingMatrix, ScalingRenderer};
use crate::{Error, Pixels, PixelsContext, ScalingMode, SurfaceSize, SurfaceTexture, TextureError};

/// A builder to help create customized pixel buffers.
pub struct PixelsBuilder<'req, 'dev, 'win, W: wgpu::WindowHandle + 'win> {
    request_adapter_options: Option<wgpu::RequestAdapterOptions<'req, 'win>>,
    device_descriptor: Option<wgpu::DeviceDescriptor<'dev>>,
    backend: wgpu::Backends,
    width: u32,
    height: u32,
    _pixel_aspect_ratio: f64,
    present_mode: wgpu::PresentMode,
    surface_texture: SurfaceTexture<W>,
    texture_format: wgpu::TextureFormat,
    render_texture_format: Option<wgpu::TextureFormat>,
    surface_texture_format: Option<wgpu::TextureFormat>,
    clear_color: wgpu::Color,
    blend_state: wgpu::BlendState,
    alpha_mode: wgpu::CompositeAlphaMode,
}

impl<'req, 'dev, 'win, W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'win>
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
    /// # let window = pixels_mocks::Window;
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
    pub fn new(width: u32, height: u32, surface_texture: SurfaceTexture<W>) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        Self {
            request_adapter_options: None,
            device_descriptor: None,
            backend: wgpu::Backends::from_env().unwrap_or_else(wgpu::Backends::all),
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
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        }
    }

    /// Add options for requesting a [`wgpu::Adapter`].
    pub fn request_adapter_options(
        mut self,
        request_adapter_options: wgpu::RequestAdapterOptions<'req, 'win>,
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

    /// Optimistically set the `wgpu` present mode.
    ///
    /// This differs from [`PixelsBuilder::enable_vsync`] by allowing the present mode to be set to
    /// any value.
    ///
    /// When the [`wgpu::Surface`] is created on [`PixelsBuilder::build`], if this present mode is
    /// not supported, Pixels defaults to `AutoVsync` which is supported on all platforms.
    /// After building, [`PixelsContext::surface_capabilities`] can be used to check the surface's
    /// supported present modes.
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
    /// to. The chosen format can be retrieved later with [`Pixels::surface_texture_format`].
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
    /// # let window = pixels_mocks::Window;
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

    /// Sets the alpha mode.
    ///
    /// Default value is `Auto`.
    ///
    /// ```no_run
    /// use pixels::wgpu::CompositeAlphaMode;
    ///
    /// # use pixels::PixelsBuilder;
    /// # let window = pixels_mocks::Window;
    /// # let surface_texture = pixels::SurfaceTexture::new(320, 240, &window);
    /// let mut pixels = PixelsBuilder::new(320, 240, surface_texture)
    ///     .alpha_mode(CompositeAlphaMode::PostMultiplied)
    ///     .build()?;
    /// # Ok::<(), pixels::Error>(())
    /// ```
    pub fn alpha_mode(mut self, alpha_mode: wgpu::CompositeAlphaMode) -> Self {
        self.alpha_mode = alpha_mode;
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
    /// # let window = pixels_mocks::Window;
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
    async fn build_impl(self) -> Result<Pixels<'win>, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: self.backend,
            ..wgpu::InstanceDescriptor::new_without_display_handle().with_env()
        });

        // TODO: Use `options.pixel_aspect_ratio` to stretch the scaled texture
        let surface = instance.create_surface(self.surface_texture.window)?;
        let compatible_surface = Some(&surface);
        let request_adapter_options = &self.request_adapter_options;
        let adapter = match wgpu::util::initialize_adapter_from_env(&instance, compatible_surface)
            .await
        {
            Ok(adapter) => Ok(adapter),
            Err(_) => {
                instance
                    .request_adapter(&request_adapter_options.as_ref().map_or_else(
                        || wgpu::RequestAdapterOptions {
                            compatible_surface,
                            force_fallback_adapter: false,
                            power_preference: wgpu::PowerPreference::from_env().unwrap_or_default(),
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

        let adapter = adapter.map_err(|_| Error::AdapterNotFound)?;

        let device_descriptor = self
            .device_descriptor
            .unwrap_or_else(|| wgpu::DeviceDescriptor {
                required_limits: adapter.limits(),
                ..wgpu::DeviceDescriptor::default()
            });

        let (device, queue) = adapter.request_device(&device_descriptor).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let present_mode_supported = match self.present_mode {
            wgpu::PresentMode::AutoVsync | wgpu::PresentMode::AutoNoVsync => true,
            _ => surface_capabilities
                .present_modes
                .contains(&self.present_mode),
        };
        let present_mode = if present_mode_supported {
            self.present_mode
        } else {
            wgpu::PresentMode::AutoVsync
        };
        if self.alpha_mode != wgpu::CompositeAlphaMode::Auto
            && !surface_capabilities.alpha_modes.contains(&self.alpha_mode)
        {
            return Err(Error::InvalidAlphaMode(self.alpha_mode));
        }

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
        let scaling_mode = ScalingMode::PixelPerfect;
        let BackingTexture {
            scaling_matrix_inverse,
            texture_extent,
            texture,
            scaling_renderer,
            buffer_layout,
        } = create_backing_texture(
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
            scaling_mode,
        )?;

        // Create the pixel buffer
        let pixels = vec![0; buffer_layout.size()];

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
            surface_capabilities,
        };

        let alpha_mode = self.alpha_mode;
        let pixels = Pixels {
            context,
            adapter,
            surface_size,
            present_mode,
            render_texture_format,
            surface_texture_format,
            blend_state,
            pixels,
            buffer_layout,
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
    pub fn build(self) -> Result<Pixels<'win>, Error> {
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
    /// # let window = pixels_mocks::Window;
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
    pub async fn build_async(self) -> Result<Pixels<'win>, Error> {
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

/// GPU resources backing the pixel buffer.
pub(crate) struct BackingTexture {
    pub(crate) scaling_matrix_inverse: ultraviolet::Mat4,
    pub(crate) texture_extent: wgpu::Extent3d,
    pub(crate) texture: wgpu::Texture,
    pub(crate) scaling_renderer: ScalingRenderer,
    pub(crate) buffer_layout: BufferLayout,
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
    scaling_mode: ScalingMode,
) -> Result<BackingTexture, TextureError> {
    check_texture_size(device, width, height)?;
    let buffer_layout = BufferLayout::new(backing_texture_format, width, height)?;

    let scaling_matrix_inverse = ScalingMatrix::new(
        (width as f32, height as f32),
        (surface_size.width as f32, surface_size.height as f32),
        scaling_mode,
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
        scaling_mode,
    );

    Ok(BackingTexture {
        scaling_matrix_inverse,
        texture_extent,
        texture,
        scaling_renderer,
        buffer_layout,
    })
}

/// Memory layout of the CPU-side pixel buffer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BufferLayout {
    /// Bytes per row of texel blocks.
    pub(crate) bytes_per_row: u32,
    /// Rows of texel blocks.
    pub(crate) rows_per_image: u32,
}

impl BufferLayout {
    /// # Errors
    ///
    /// [`TextureError::UnsupportedFormat`] when the format has no well-defined copy size.
    pub(crate) fn new(
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Result<Self, TextureError> {
        let block_size = format
            .block_copy_size(None)
            .ok_or(TextureError::UnsupportedFormat(format))?;
        let (block_width, block_height) = format.block_dimensions();

        Ok(Self {
            bytes_per_row: width.div_ceil(block_width) * block_size,
            rows_per_image: height.div_ceil(block_height),
        })
    }

    /// Total size in bytes.
    pub(crate) fn size(&self) -> usize {
        self.bytes_per_row as usize * self.rows_per_image as usize
    }
}

/// Average bytes per texel; `0.0` for formats rejected by [`BufferLayout::new`].
fn texture_format_size(format: wgpu::TextureFormat) -> f32 {
    let (block_width, block_height) = format.block_dimensions();
    format.block_copy_size(None).map_or(0.0, |block_size| {
        block_size as f32 / (block_width * block_height) as f32
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::{AstcBlock, AstcChannel, TextureFormat};

    #[test]
    fn buffer_layout_uncompressed() {
        let layout = BufferLayout::new(TextureFormat::Rgba8UnormSrgb, 320, 240).unwrap();
        assert_eq!(layout.bytes_per_row, 320 * 4);
        assert_eq!(layout.rows_per_image, 240);
        assert_eq!(layout.size(), 320 * 240 * 4);
    }

    #[test]
    fn buffer_layout_large_odd_size_is_exact() {
        // `5001 * 4001` is not exactly representable as `f32`.
        let layout = BufferLayout::new(TextureFormat::Rgba8UnormSrgb, 5001, 4001).unwrap();
        assert_eq!(layout.size(), 5001 * 4001 * 4);
    }

    #[test]
    fn buffer_layout_compressed() {
        let layout = BufferLayout::new(TextureFormat::Bc1RgbaUnorm, 64, 32).unwrap();
        assert_eq!(layout.bytes_per_row, 16 * 8);
        assert_eq!(layout.rows_per_image, 8);

        let astc = TextureFormat::Astc {
            block: AstcBlock::B8x8,
            channel: AstcChannel::Unorm,
        };
        let layout = BufferLayout::new(astc, 64, 64).unwrap();
        assert_eq!(layout.size(), 8 * 8 * 16);
        assert_eq!(texture_format_size(astc), 0.25);
    }

    #[test]
    fn buffer_layout_unsupported_format() {
        assert!(matches!(
            BufferLayout::new(TextureFormat::Depth24Plus, 16, 16),
            Err(TextureError::UnsupportedFormat(TextureFormat::Depth24Plus))
        ));
    }
}
