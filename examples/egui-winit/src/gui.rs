use chrono::Timelike;
use egui::{FontDefinitions, PaintJobs};
use egui_demo_lib::WrapApp;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::backend::{AppOutput, FrameBuilder};
use epi::App;
use pixels::{wgpu, PixelsContext};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::event_loop::EventLoopProxy;

/// A custom event type for winit.
pub(crate) enum GuiEvent {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
pub(crate) struct ExampleRepaintSignal(std::sync::Mutex<EventLoopProxy<GuiEvent>>);

impl epi::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(GuiEvent::RequestRedraw)
            .ok();
    }
}

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Gui {
    // State for egui.
    start_time: Instant,
    platform: Platform,
    screen_descriptor: ScreenDescriptor,
    repaint_signal: Arc<ExampleRepaintSignal>,
    rpass: RenderPass,
    paint_jobs: PaintJobs,

    // State for the demo app.
    app: WrapApp,
    previous_frame_time: Option<f32>,
}

impl Gui {
    /// Create egui.
    pub(crate) fn new(
        event_loop_proxy: EventLoopProxy<GuiEvent>,
        width: u32,
        height: u32,
        scale_factor: f64,
        context: &PixelsContext,
    ) -> Self {
        let platform = Platform::new(PlatformDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor: scale_factor as f32,
        };
        let repaint_signal = Arc::new(ExampleRepaintSignal(Mutex::new(event_loop_proxy)));
        let rpass = RenderPass::new(&context.device, wgpu::TextureFormat::Bgra8UnormSrgb);
        let app = WrapApp::default();

        Self {
            start_time: Instant::now(),
            platform,
            screen_descriptor,
            repaint_signal,
            rpass,
            paint_jobs: Vec::new(),
            app,
            previous_frame_time: None,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::Event<'_, GuiEvent>) {
        self.platform.handle_event(event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.screen_descriptor.physical_width = width;
        self.screen_descriptor.physical_height = height;
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.scale_factor = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self) {
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());

        // Begin the egui frame.
        let start = Instant::now();
        self.platform.begin_frame();
        let mut app_output = AppOutput::default();
        let mut frame = FrameBuilder {
            info: epi::IntegrationInfo {
                web_info: None,
                cpu_usage: self.previous_frame_time,
                seconds_since_midnight: Some(seconds_since_midnight()),
                native_pixels_per_point: Some(self.screen_descriptor.scale_factor),
            },
            tex_allocator: Some(&mut self.rpass),
            output: &mut app_output,
            repaint_signal: self.repaint_signal.clone(),
        }
        .build();

        // Draw the demo application.
        self.app.update(&self.platform.context(), &mut frame);

        // End the egui frame and create all paint jobs to prepare for rendering.
        let (_output, paint_commands) = self.platform.end_frame();
        self.paint_jobs = self.platform.context().tessellate(paint_commands);

        // Update timing info for CPU usage display in the demo.
        let frame_time = Instant::now().duration_since(start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        self.rpass.update_texture(
            &context.device,
            &context.queue,
            &self.platform.context().texture(),
        );
        self.rpass
            .update_user_textures(&context.device, &context.queue);
        self.rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Record all render passes.
        self.rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        );
    }
}

/// Time of day as seconds since midnight. Used for clock in the demo app.
pub fn seconds_since_midnight() -> f64 {
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}
