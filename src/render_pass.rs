use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wgpu::{Extent3d, TextureView};

/// A reference-counted [`wgpu::Device`]
pub type Device = Rc<wgpu::Device>;

/// A reference-counted [`wgpu::Queue`] (with interior mutability)
pub type Queue = Rc<RefCell<wgpu::Queue>>;

/// The boxed render pass type for dynamic dispatch
pub type BoxedRenderPass = Box<dyn RenderPass>;

/// Objects that implement this trait can be added to [`Pixels`] as a render pass.
///
/// [`Pixels`] always has at least one render pass; a scaling pass that uses a nearest-neighbor
/// sampler to preserve pixel edges. Optionally it may also have a second scaling pass that
/// transforms the texture to its final size (for non-square pixel aspect ratios). During this
/// second pass, the texture is stretched horizontally using a linear sampler.
///
/// Any additional render passes are executed afterward.
///
/// Each render pass is configured with one [`wgpu::TextureView`] as an input. You will probably
/// want to create a binding for this `texture_view` so your shaders can sample from it.
///
/// The render pass will also receive a reference to another [`wgpu::TextureView`] when the pass is
/// executed. This texture view is the `render_target`.
///
/// [`Pixels`]: ./struct.Pixels.html
pub trait RenderPass {
    /// Called when it is time to execute this render pass. Use the `encoder` to encode all
    /// commands related to this render pass. The result must be stored to the `render_target`.
    ///
    /// # Arguments
    /// * `encoder` - Command encoder for the render pass
    /// * `render_target` - A reference to the output texture
    /// * `texels` - The byte slice passed to `Pixels::render`
    fn render(&self, encoder: &mut wgpu::CommandEncoder, render_target: &TextureView);

    /// This method will be called when the input [`wgpu::TextureView`] needs to be rebinded.
    ///
    /// A [`wgpu::TextureView`] is provided to the `RenderPass` factory as an input texture with
    /// the original [`SurfaceTexture`] size. This method is called in response to resizing the
    /// [`SurfaceTexture`], where your `RenderPass` impl can update its input texture for the new
    /// size.
    ///
    /// # Arguments
    /// * `input_texture` - A reference to the `TextureView` for this render pass's input
    /// * `input_texture_size` - The `input_texture` size
    ///
    /// [`Pixels`]: ./struct.Pixels.html
    /// [`SurfaceTexture`]: ./struct.SurfaceTexture.html
    fn update_bindings(&mut self, input_texture: &TextureView, input_texture_size: &Extent3d);

    /// When the window is resized, this method will be called, allowing the render pass to
    /// customize itself to the display size.
    ///
    /// The default implementation is a no-op.
    ///
    /// # Arguments
    /// * `encoder` - Command encoder for the render pass
    /// * `width` - Render target width in physical pixel units
    /// * `height` - Render target height in physical pixel units
    #[allow(unused_variables)]
    fn resize(&mut self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {}

    /// This function implements [`Debug`](fmt::Debug) for trait objects.
    ///
    /// You are encouraged to override the default impl to provide better debug messages.
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn RenderPass")
    }
}

impl fmt::Debug for dyn RenderPass + 'static {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}
