#![deny(clippy::all)]
#![forbid(unsafe_code)]

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

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

struct Display<'win> {
    window: Arc<Window>,
    pixels: Pixels<'win>,
}

fn _main(event_loop: EventLoop<()>) {
    let mut display: Option<Display> = None;

    let mut world = World::new();

    let res = event_loop.run(|event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);
        match event {
            Event::Resumed => {
                let window = Arc::new(Window::new(elwt).unwrap());
                let pixels = {
                    let window_size = window.inner_size();
                    let surface_texture = SurfaceTexture::new(
                        window_size.width,
                        window_size.height,
                        Arc::clone(&window),
                    );
                    Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
                };
                window.request_redraw();
                display = Some(Display { window, pixels });
            }
            Event::Suspended => {
                display = None;
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                if let Some(display) = &mut display {
                    world.draw(display.pixels.frame_mut());
                    display.pixels.render().unwrap();
                    display.window.request_redraw();
                }
            }
            _ => {}
        }
        if display.is_some() {
            world.update();
        }
    });
    res.unwrap();
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
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
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

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));
    let event_loop = EventLoopBuilder::new().with_android_app(app).build();
    log::info!("Hello from android!");
    _main(event_loop);
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) // Default Log Level
        .parse_default_env()
        .init();
    let event_loop = EventLoop::new().unwrap();
    log::info!("Hello from desktop!");
    _main(event_loop);
}
