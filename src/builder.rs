use crate::renderers::{ScalingMatrix, ScalingRenderer};
use crate::{Error, Pixels, PixelsContext, SurfaceTexture};
use raw_window_handle::HasRawWindowHandle;
use std::env;

/// A builder to help create customized pixel buffers.
pub struct PixelsBuilder<'req, 'win, W: HasRawWindowHandle> {
    request_adapter_options: Option<wgpu::RequestAdapterOptions<'req>>,
    device_descriptor: wgpu::DeviceDescriptor,
    backend: wgpu::BackendBit,
    width: u32,
    height: u32,
    pixel_aspect_ratio: f64,
    present_mode: wgpu::PresentMode,
    surface_texture: SurfaceTexture<'win, W>,
    texture_format: wgpu::TextureFormat,
    render_texture_format: wgpu::TextureFormat,
}

impl<'req, 'win, W: HasRawWindowHandle> PixelsBuilder<'req, 'win, W> {
    /// Create a builder that can be finalized into a [`Pixels`] pixel buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pixels::PixelsBuilder;
    /// # let window = pixels_mocks::RWH;
    /// # let surface_texture = pixels::SurfaceTexture::new(1024, 768, &window);
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
    pub fn new(
        width: u32,
        height: u32,
        surface_texture: SurfaceTexture<'win, W>,
    ) -> PixelsBuilder<'req, 'win, W> {
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
            render_texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
        }
    }

    /// Add options for requesting a [`wgpu::Adapter`].
    pub fn request_adapter_options(
        mut self,
        request_adapter_options: wgpu::RequestAdapterOptions<'req>,
    ) -> PixelsBuilder<'req, 'win, W> {
        self.request_adapter_options = Some(request_adapter_options);
        self
    }

    /// Add options for requesting a [`wgpu::Device`].
    pub fn device_descriptor(
        mut self,
        device_descriptor: wgpu::DeviceDescriptor,
    ) -> PixelsBuilder<'req, 'win, W> {
        self.device_descriptor = device_descriptor;
        self
    }

    /// Set which backends wgpu will attempt to use.
    ///
    /// The default value of this is [`wgpu::BackendBit::PRIMARY`], which enables
    /// the well supported backends for wgpu.
    pub fn wgpu_backend(mut self, backend: wgpu::BackendBit) -> PixelsBuilder<'req, 'win, W> {
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
    pub fn pixel_aspect_ratio(mut self, pixel_aspect_ratio: f64) -> PixelsBuilder<'req, 'win, W> {
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
    pub fn enable_vsync(mut self, enable_vsync: bool) -> PixelsBuilder<'req, 'win, W> {
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
    pub fn present_mode(mut self, present_mode: wgpu::PresentMode) -> PixelsBuilder<'req, 'win, W> {
        self.present_mode = present_mode;
        self
    }

    /// Set the texture format.
    ///
    /// The default value is [`wgpu::TextureFormat::Rgba8UnormSrgb`], which is 4 unsigned bytes in
    /// `RGBA` order using the SRGB color space. This is typically what you want when you are
    /// working with color values from popular image editing tools or web apps.
    pub fn texture_format(
        mut self,
        texture_format: wgpu::TextureFormat,
    ) -> PixelsBuilder<'req, 'win, W> {
        self.texture_format = texture_format;
        self
    }

    /// Set the render texture format.
    ///
    /// The default value is [`wgpu::TextureFormat::Bgra8UnormSrgb`], which is 4 unsigned bytes in
    /// `BGRA` order using the SRGB color space. This format depends on the hardware/platform the
    /// pixel buffer is rendered to/for.
    pub fn render_texture_format(
        mut self,
        texture_format: wgpu::TextureFormat,
    ) -> PixelsBuilder<'req, 'win, W> {
        self.render_texture_format = texture_format;
        self
    }

    /// Create a pixel buffer from the options builder.
    ///
    /// # Errors
    ///
    /// Returns an error when a [`wgpu::Adapter`] cannot be found.
    pub fn build(self) -> Result<Pixels, Error> {
        let instance = wgpu::Instance::new(self.backend);

        // TODO: Use `options.pixel_aspect_ratio` to stretch the scaled texture
        let surface = unsafe { instance.create_surface(self.surface_texture.window) };
        let compatible_surface = Some(&surface);
        let adapter = instance.request_adapter(&self.request_adapter_options.map_or_else(
            || wgpu::RequestAdapterOptions {
                compatible_surface,
                power_preference: get_default_power_preference(),
            },
            |rao| wgpu::RequestAdapterOptions {
                compatible_surface: rao.compatible_surface.or(compatible_surface),
                power_preference: rao.power_preference,
            },
        ));
        let adapter = pollster::block_on(adapter).ok_or(Error::AdapterNotFound)?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&self.device_descriptor, None))
                .map_err(Error::DeviceNotFound)?;

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
            label: Some("pixels_source_texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_format_size = get_texture_format_size(self.texture_format);

        // Create the pixel buffer
        let capacity = ((width * height) as f32 * texture_format_size) as usize;
        let mut pixels = Vec::with_capacity(capacity);
        pixels.resize_with(capacity, Default::default);

        let present_mode = self.present_mode;

        // Create swap chain
        let surface_size = self.surface_texture.size;
        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: self.render_texture_format,
                width: surface_size.width,
                height: surface_size.height,
                present_mode,
            },
        );

        let scaling_matrix_inverse = ScalingMatrix::new(
            (width as f32, height as f32),
            (surface_size.width as f32, surface_size.height as f32),
        )
        .transform
        .inversed();

        let scaling_renderer = ScalingRenderer::new(
            &device,
            &texture_view,
            &texture_extent,
            self.render_texture_format,
        );

        let context = PixelsContext {
            device,
            queue,
            surface,
            swap_chain,
            texture,
            texture_extent,
            texture_format: self.texture_format,
            texture_format_size,
            scaling_renderer,
        };
        let mut pixels = Pixels {
            context,
            surface_size,
            present_mode,
            pixels,
            scaling_matrix_inverse,
            render_texture_format: self.render_texture_format,
        };
        create_swap_chain(&mut pixels);

        Ok(pixels)
    }
}

pub(crate) fn create_swap_chain(pixels: &mut Pixels) {
    pixels.context.swap_chain = pixels.context.device.create_swap_chain(
        &pixels.context.surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: pixels.render_texture_format,
            width: pixels.surface_size.width,
            height: pixels.surface_size.height,
            present_mode: pixels.present_mode,
        },
    );
}

fn get_texture_format_size(texture_format: wgpu::TextureFormat) -> f32 {
    match texture_format {
        // 8-bit formats
        wgpu::TextureFormat::R8Unorm
        | wgpu::TextureFormat::R8Snorm
        | wgpu::TextureFormat::R8Uint
        | wgpu::TextureFormat::R8Sint => 1.0,

        // 16-bit formats
        wgpu::TextureFormat::R16Uint
        | wgpu::TextureFormat::R16Sint
        | wgpu::TextureFormat::R16Float
        | wgpu::TextureFormat::Rg8Unorm
        | wgpu::TextureFormat::Rg8Snorm
        | wgpu::TextureFormat::Rg8Uint
        | wgpu::TextureFormat::Rg8Sint => 2.0,

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
        | wgpu::TextureFormat::Depth24PlusStencil8 => 4.0,

        // 64-bit formats
        wgpu::TextureFormat::Rg32Uint
        | wgpu::TextureFormat::Rg32Sint
        | wgpu::TextureFormat::Rg32Float
        | wgpu::TextureFormat::Rgba16Uint
        | wgpu::TextureFormat::Rgba16Sint
        | wgpu::TextureFormat::Rgba16Float => 8.0,

        // 128-bit formats
        wgpu::TextureFormat::Rgba32Uint
        | wgpu::TextureFormat::Rgba32Sint
        | wgpu::TextureFormat::Rgba32Float => 16.0,

        // Compressed formats
        wgpu::TextureFormat::Bc1RgbaUnorm
        | wgpu::TextureFormat::Bc1RgbaUnormSrgb
        | wgpu::TextureFormat::Bc4RUnorm
        | wgpu::TextureFormat::Bc4RSnorm => 0.5,

        wgpu::TextureFormat::Bc2RgbaUnorm
        | wgpu::TextureFormat::Bc2RgbaUnormSrgb
        | wgpu::TextureFormat::Bc3RgbaUnorm
        | wgpu::TextureFormat::Bc3RgbaUnormSrgb
        | wgpu::TextureFormat::Bc5RgUnorm
        | wgpu::TextureFormat::Bc5RgSnorm
        | wgpu::TextureFormat::Bc6hRgbUfloat
        | wgpu::TextureFormat::Bc6hRgbSfloat
        | wgpu::TextureFormat::Bc7RgbaUnorm
        | wgpu::TextureFormat::Bc7RgbaUnormSrgb => 1.0,
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
