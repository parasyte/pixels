#![deny(clippy::all)]
#![forbid(unsafe_code)]

use game_loop::{game_loop, Time, TimeTrait as _};
use gilrs::{Button, GamepadId, Gilrs};
use log::{debug, error};
use pixels::{Error, Pixels, SurfaceTexture};
use simple_invaders::{Controls, Direction, World, FPS, HEIGHT, TIME_STEP, WIDTH};
use std::{env, time::Duration};
use winit::{
    dpi::LogicalSize, event::VirtualKeyCode, event_loop::EventLoop, window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

/// Uber-struct representing the entire game.
struct Game {
    /// Software renderer.
    pixels: Pixels,
    /// Invaders world.
    world: World,
    /// Player controls for world updates.
    controls: Controls,
    /// Event manager.
    input: WinitInputHelper,
    /// GamePad manager.
    gilrs: Gilrs,
    /// GamePad ID for the player.
    gamepad: Option<GamepadId>,
    /// Game pause state.
    paused: bool,
}

impl Game {
    fn new(pixels: Pixels, debug: bool) -> Self {
        Self {
            pixels,
            world: World::new(generate_seed(), debug),
            controls: Controls::default(),
            input: WinitInputHelper::new(),
            gilrs: Gilrs::new().unwrap(), // XXX: Don't unwrap.
            gamepad: None,
            paused: false,
        }
    }

    fn update_controls(&mut self) {
        // Pump the gilrs event loop and find an active gamepad
        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            let pad = self.gilrs.gamepad(id);
            if self.gamepad.is_none() {
                debug!("Gamepad with id {} is connected: {}", id, pad.name());
                self.gamepad = Some(id);
            } else if event == gilrs::ev::EventType::Disconnected {
                debug!("Gamepad with id {} is disconnected: {}", id, pad.name());
                self.gamepad = None;
            }
        }

        self.controls = {
            // Keyboard controls
            let mut left = self.input.key_held(VirtualKeyCode::Left);
            let mut right = self.input.key_held(VirtualKeyCode::Right);
            let mut fire = self.input.key_pressed(VirtualKeyCode::Space);
            let mut pause = self.input.key_pressed(VirtualKeyCode::Pause)
                | self.input.key_pressed(VirtualKeyCode::P);

            // GamePad controls
            if let Some(id) = self.gamepad {
                let gamepad = self.gilrs.gamepad(id);

                left |= gamepad.is_pressed(Button::DPadLeft);
                right |= gamepad.is_pressed(Button::DPadRight);
                fire |= gamepad.button_data(Button::South).map_or(false, |button| {
                    button.is_pressed() && button.counter() == self.gilrs.counter()
                });
                pause |= gamepad.button_data(Button::Start).map_or(false, |button| {
                    button.is_pressed() && button.counter() == self.gilrs.counter()
                });
            }
            self.gilrs.inc();

            if pause {
                self.paused = !self.paused;
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
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();

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

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let game = Game::new(pixels, debug);

    game_loop(
        event_loop,
        window,
        game,
        FPS as u32,
        0.1,
        move |g| {
            // Update the world
            if !g.game.paused {
                g.game.world.update(&g.game.controls);
            }
        },
        move |g| {
            // Drawing
            g.game.world.draw(g.game.pixels.get_frame_mut());
            if let Err(err) = g.game.pixels.render() {
                error!("pixels.render() failed: {err}");
                g.exit();
            }

            // Sleep the main thread to limit drawing to the fixed time step.
            // See: https://github.com/parasyte/pixels/issues/174
            let dt = TIME_STEP.as_secs_f64() - Time::now().sub(&g.current_instant());
            if dt > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(dt));
            }
        },
        |g, event| {
            // Let winit_input_helper collect events to build its state.
            if g.game.input.update(event) {
                // Update controls
                g.game.update_controls();

                // Close events
                if g.game.input.key_pressed(VirtualKeyCode::Escape) || g.game.input.quit() {
                    g.exit();
                    return;
                }

                // Resize the window
                if let Some(size) = g.game.input.window_resized() {
                    if let Err(err) = g.game.pixels.resize_surface(size.width, size.height) {
                        error!("pixels.resize_surface() failed: {err}");
                        g.exit();
                    }
                }
            }
        },
    );
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
