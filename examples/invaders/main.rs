use std::env;
use std::time::Instant;

use pixels::{Error, Pixels, SurfaceTexture};
use simple_invaders::{Controls, Direction, World, SCREEN_HEIGHT, SCREEN_WIDTH};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    // Enable debug mode with `DEBUG=true` environment variable
    let debug = env::var("DEBUG")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    let (window, surface, width, height, mut hidpi_factor) = {
        let scale = 3.0;
        let width = SCREEN_WIDTH as f64 * scale;
        let height = SCREEN_HEIGHT as f64 * scale;

        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .with_title("pixel invaders")
            .build(&event_loop)
            .unwrap();
        let surface = pixels::wgpu::Surface::create(&window);
        let hidpi_factor = window.hidpi_factor();
        let size = window.inner_size().to_physical(hidpi_factor);

        (
            window,
            surface,
            size.width.round() as u32,
            size.height.round() as u32,
            hidpi_factor,
        )
    };

    let surface_texture = SurfaceTexture::new(width, height, surface);
    let mut pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?;
    let mut invaders = World::new(debug);
    let mut time = Instant::now();
    let mut controls = Controls::default();

    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                invaders.draw(pixels.get_frame());
                pixels.render();
            }
            _ => (),
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Keyboard controls
            controls.direction = if input.key_held(VirtualKeyCode::Left) {
                Direction::Left
            } else if input.key_held(VirtualKeyCode::Right) {
                Direction::Right
            } else {
                Direction::Still
            };
            controls.fire = input.key_pressed(VirtualKeyCode::Space);

            // Adjust high DPI factor
            if let Some(factor) = input.hidpi_changed() {
                hidpi_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                let size = size.to_physical(hidpi_factor);
                let width = size.width.round() as u32;
                let height = size.height.round() as u32;

                pixels.resize(width, height);
            }

            // Get a new delta time.
            let now = Instant::now();
            let dt = now.duration_since(time);
            time = now;

            // Update the game logic and request redraw
            invaders.update(&dt, &controls);
            window.request_redraw();
        }
    });
}
