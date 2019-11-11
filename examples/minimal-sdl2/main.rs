#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::*;
use pixels::{wgpu::Surface, Pixels, SurfaceTexture};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

fn main() -> Result<(), String> {
    env_logger::init();
    let sdl = beryllium::init()?;
    let window = sdl.create_window(
        "Hello Pixels",
        WINDOW_POSITION_CENTERED,
        WINDOW_POSITION_CENTERED,
        WIDTH as i32,
        HEIGHT as i32,
        WindowFlags::default(),
    )?;
    window.set_minimum_size(WIDTH as i32, HEIGHT as i32);
    window.set_resizable(true);

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, surface);
        Pixels::new(WIDTH, HEIGHT, surface_texture).map_err(|e| format!("{:?}", e))?
    };
    let mut world = World::new();

    'game_loop: loop {
        while let Some(event) = sdl.poll_event() {
            match event {
                // Close events
                Event::Quit { .. } => break 'game_loop,
                Event::Keyboard {
                    key_info:
                        KeyInfo {
                            keycode: Some(key), ..
                        },
                    ..
                } if key == Keycode::Escape => break 'game_loop,

                // Resize the window
                Event::WindowSizeChanged { width, height, .. } => {
                    pixels.resize(width as u32, height as u32)
                }

                _ => (),
            }
        }

        // Update internal state
        world.update();

        // Draw the current frame
        world.draw(pixels.get_frame());
        pixels.render();
    }

    Ok(())
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE - 1 >= WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE - 1 >= HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
