#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, PixelsBuilder, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

// The circle is actually defined as an ellipse with minor axis 56 pixels and major axis 64 pixels.
const CIRCLE_AXES: (i16, i16) = (56, 64);
const CIRCLE_SEMI: (i16, i16) = (CIRCLE_AXES.0 / 2, CIRCLE_AXES.1 / 2);

// The Pixel Aspect Ratio is the difference between the physical width and height of a single pixel.
// For most users, this ratio will be 1:1, i.e. the value will be `1.0`. Some devices display
// non-square pixels, and the pixel aspect ratio can simulate this difference on devices with square
// pixels. In this example, the ellipse will be rendered as a circle if it is drawn with a pixel
// aspect ratio of 8:7.
const PAR: f32 = 8.0 / 7.0;

/// Representation of the application state. In this example, a circle will bounce around the screen.
struct World {
    circle_x: i16,
    circle_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        // The window size is horizontally stretched by the PAR.
        let size = LogicalSize::new(WIDTH as f64 * PAR as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixel Aspect Ratio")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .pixel_aspect_ratio(PAR)
            .build()?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving circle.
    fn new() -> Self {
        Self {
            circle_x: CIRCLE_SEMI.0 + 24,
            circle_y: CIRCLE_SEMI.1 + 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the circle around the screen.
    fn update(&mut self) {
        if self.circle_x - CIRCLE_SEMI.0 <= 0 || self.circle_x + CIRCLE_SEMI.0 > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.circle_y - CIRCLE_SEMI.1 <= 0 || self.circle_y + CIRCLE_SEMI.1 > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.circle_x += self.velocity_x;
        self.circle_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;
            let length = {
                let x = (x - self.circle_x) as f64;
                let y = (y - self.circle_y) as f64;
                let semi_minor = (CIRCLE_SEMI.0 as f64).powf(2.0);
                let semi_major = (CIRCLE_SEMI.1 as f64).powf(2.0);

                x.powf(2.0) / semi_minor + y.powf(2.0) / semi_major
            };
            let inside_the_circle = length < 1.0;

            let rgba = if inside_the_circle {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
