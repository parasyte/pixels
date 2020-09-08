use pixels::{raw_window_handle::HasRawWindowHandle, wgpu, PixelsContext};
use std::time::Instant;

/// Manages all state required for rendering Dear ImGui over `Pixels`.
pub(crate) struct Gui {
    pub(crate) imgui: imgui::Context,
    pub(crate) platform: imgui_winit_support::WinitPlatform,
    pub(crate) renderer: imgui_wgpu::Renderer,

    last_frame: Instant,
    last_cursor: Option<imgui::MouseCursor>,
    about_open: bool,
}

impl Gui {
    /// Create Dear ImGui.
    pub(crate) fn new<W: HasRawWindowHandle>(
        window: &winit::window::Window,
        pixels: &pixels::Pixels<W>,
    ) -> Self {
        // Create Dear ImGui context
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        // Initialize winit platform support
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );

        // Configure Dear ImGui fonts
        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        // Fix incorrect colors with sRGB framebuffer
        let style = imgui.style_mut();
        for color in 0..style.colors.len() {
            style.colors[color] = gamma_to_linear(style.colors[color]);
        }

        // Create Dear ImGui WGPU renderer
        let device = pixels.device();
        let queue = pixels.queue();
        let texture_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let renderer = imgui_wgpu::Renderer::new(&mut imgui, &device, &queue, texture_format);

        // Return GUI context
        Self {
            imgui,
            platform,
            renderer,

            last_frame: Instant::now(),
            last_cursor: None,
            about_open: true,
        }
    }

    /// Prepare Dear ImGui.
    pub(crate) fn prepare(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<(), winit::error::ExternalError> {
        // Prepare Dear ImGui
        let io = self.imgui.io_mut();
        self.last_frame = io.update_delta_time(self.last_frame);
        self.platform.prepare_frame(io, window)
    }

    /// Render Dear ImGui.
    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) -> imgui_wgpu::RendererResult<()> {
        // Start a new Dear ImGui frame and update the cursor
        let ui = self.imgui.frame();

        let mouse_cursor = ui.mouse_cursor();
        if self.last_cursor != mouse_cursor {
            self.last_cursor = mouse_cursor;
            self.platform.prepare_render(&ui, window);
        }

        // Draw windows and GUI elements here
        ui.show_about_window(&mut self.about_open);

        // Render Dear ImGui with WGPU
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.renderer
            .render(ui.render(), &context.queue, &context.device, &mut rpass)
    }
}

fn gamma_to_linear(color: [f32; 4]) -> [f32; 4] {
    const GAMMA: f32 = 2.2;

    let x = color[0].powf(GAMMA);
    let y = color[1].powf(GAMMA);
    let z = color[2].powf(GAMMA);
    let w = 1.0 - (1.0 - color[3]).powf(GAMMA);
    [x, y, z, w]
}
