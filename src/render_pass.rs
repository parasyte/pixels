use wgpu::TextureView;

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
    /// The `update_bindings` method will be called when the `texture_view` needs to be recreated.
    ///
    /// It's always called at least once when the default [`Pixels`] render passes are created.
    /// This can also happen in response to resizing the [`SurfaceTexture`].
    ///
    /// You will typically recreate your bindings here to reference the new input `texture_view`.
    ///
    /// [`Pixels`]: ./struct.Pixels.html
    /// [`SurfaceTexture`]: ./struct.SurfaceTexture.html
    fn update_bindings(&mut self, texture_view: &TextureView);

    /// Called when it is time to execute this render pass. Use the `encoder` to encode all
    /// commands related to this render pass. The result must be stored to the `render_target`.
    fn render_pass(&self, encoder: &mut wgpu::CommandEncoder, render_target: &TextureView);
}
