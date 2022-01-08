#![deny(clippy::all)]
#![forbid(unsafe_code)]

use gilrs::{Button, Gilrs};
use log::{debug, error};
use pixels::{Error, Pixels, SurfaceTexture};
use simple_invaders::{Controls, Direction, World, HEIGHT, WIDTH};
use std::{env, time::Instant};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
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

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
        WindowBuilder::new()
            .with_title("pixel invaders")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut invaders = World::new(generate_seed(), debug);
    let mut time = Instant::now();
    let mut gamepad = None;

    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::RedrawRequested(_) = event {
            invaders.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
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
        if input.update(&event) {
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

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
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
