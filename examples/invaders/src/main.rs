#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use game_loop::{game_loop, Time, TimeTrait as _};
use gilrs::{Button, GamepadId, Gilrs};
use log::{debug, error};
use pixels::{Error, Pixels, SurfaceTexture};
use simple_invaders::{Controls, Direction, World, FPS, HEIGHT, TIME_STEP, WIDTH};
use std::sync::Arc;
use std::{env, time::Duration};
use winit::{dpi::LogicalSize, event_loop::EventLoop, keyboard::KeyCode, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

/// Uber-struct representing the entire game.
struct Game {
    /// Software renderer.
    pixels: Pixels<'static>,
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
    fn new(pixels: Pixels<'static>, debug: bool) -> Self {
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
            let mut left = self.input.key_held(KeyCode::ArrowLeft);
            let mut right = self.input.key_held(KeyCode::ArrowRight);
            let mut fire = self.input.key_pressed(KeyCode::Space);
            let mut pause =
                self.input.key_pressed(KeyCode::Pause) | self.input.key_pressed(KeyCode::KeyP);

            // GamePad controls
            if let Some(id) = self.gamepad {
                let gamepad = self.gilrs.gamepad(id);

                left |= gamepad.is_pressed(Button::DPadLeft);
                right |= gamepad.is_pressed(Button::DPadRight);
                fire |= gamepad.button_data(Button::South).is_some_and(|button| {
                    button.is_pressed() && button.counter() == self.gilrs.counter()
                });
                pause |= gamepad.button_data(Button::Start).is_some_and(|button| {
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

    fn reset_game(&mut self) {
        self.world.reset_game();
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    // Enable debug mode with `DEBUG=true` environment variable
    let debug = env::var("DEBUG")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
        let window = WindowBuilder::new()
            .with_title("pixel invaders")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap();
        Arc::new(window)
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let game = Game::new(pixels, debug);

    let res = game_loop(
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
            g.game.world.draw(g.game.pixels.frame_mut());
            if let Err(err) = g.game.pixels.render() {
                log_error("pixels.render", err);
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
                if g.game.input.key_pressed(KeyCode::Escape) || g.game.input.close_requested() {
                    g.exit();
                    return;
                }

                // Reset game
                if g.game.input.key_pressed(KeyCode::KeyR) {
                    g.game.reset_game();
                }

                // Resize the window
                if let Some(size) = g.game.input.window_resized() {
                    if let Err(err) = g.game.pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);
                        g.exit();
                    }
                }
            }
        },
    );
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
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
