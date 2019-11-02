use std::env;
use std::time::Instant;

use gilrs::{Button, Gilrs};
use log::debug;
use pixels::{Error, Pixels, SurfaceTexture};
use simple_invaders::{Controls, Direction, World, SCREEN_HEIGHT, SCREEN_WIDTH};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let mut gilrs = Gilrs::new().unwrap();

    // Enable debug mode with `DEBUG=true` environment variable
    let debug = env::var("DEBUG")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    let (window, surface, width, height, mut hidpi_factor) =
        create_window("pixel invaders", &event_loop);
    let surface_texture = SurfaceTexture::new(width, height, surface);
    let mut pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?;
    let mut invaders = World::new(generate_seed(), debug);
    let mut time = Instant::now();
    let mut gamepad = None;

    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            invaders.draw(pixels.get_frame());
            pixels.render();
        }

        // Pump the gilrs event loop and find an active gamepad
        while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            let pad = gilrs.gamepad(id);
            if gamepad.is_none() {
                debug!("Gamepad with id {} is connected: {}", id, pad.name());
                gamepad = Some(id);
            } else if event == gilrs::ev::EventType::Disconnected {
                debug!("Gamepad with id {} is disconnected: {}", id, pad.name());
                gamepad = None;
            }
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let controls = {
                // Keyboard controls
                let mut left = input.key_held(VirtualKeyCode::Left);
                let mut right = input.key_held(VirtualKeyCode::Right);
                let mut fire = input.key_pressed(VirtualKeyCode::Space);

                // Gamepad controls
                if let Some(id) = gamepad {
                    let gamepad = gilrs.gamepad(id);

                    left = left || gamepad.is_pressed(Button::DPadLeft);
                    right = right || gamepad.is_pressed(Button::DPadRight);
                    fire = fire
                        || gamepad.button_data(Button::South).map_or(false, |button| {
                            button.is_pressed() && button.counter() == gilrs.counter()
                        });
                }

                let direction = if left {
                    Direction::Left
                } else if right {
                    Direction::Right
                } else {
                    Direction::Still
                };

                Controls { direction, fire }
            };

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

/// Create a window for the game.
///
/// Automatically scales the window to cover about 2/3 of the monitor height.
///
/// # Returns
///
/// Tuple of `(window, surface, width, height, hidpi_factor)`
/// `width` and `height` are in `LogicalSize` units.
fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, pixels::wgpu::Surface, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    let hidpi_factor = window.hidpi_factor();

    // Get dimensions
    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;
    let (monitor_width, monitor_height) = {
        let size = window.current_monitor().size();
        (size.width / hidpi_factor, size.height / hidpi_factor)
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round();

    // Resize, center, and display the window
    let min_size = PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let surface = pixels::wgpu::Surface::create(&window);
    let size = default_size.to_physical(hidpi_factor);

    (
        window,
        surface,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
}

/// Generate a pseudorandom seed for the game's PRNG.
fn generate_seed() -> (u64, u64) {
    use byteorder::{ByteOrder, NativeEndian};
    use getrandom::getrandom;

    let mut seed = [0_u8; 16];

    getrandom(&mut seed).expect("failed to getrandom");

    (
        NativeEndian::read_u64(&seed[0..8]),
        NativeEndian::read_u64(&seed[8..16]),
    )
}
