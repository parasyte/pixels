#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use pixels::{Pixels, SurfaceTexture};
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
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

fn _main(event_loop: EventLoop<()>) {
    let mut window: Option<Window> = None;
    let mut pixels: Option<Pixels> = None;

    let mut world = World::new();

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::Resumed => {
                let _window = Window::new(event_loop).unwrap();
                let _pixels = {
                    let window_size = _window.inner_size();
                    let surface_texture =
                        SurfaceTexture::new(window_size.width, window_size.height, &_window);
                    Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
                };
                _window.request_redraw();
                window = Some(_window);
                pixels = Some(_pixels);
            }
            Event::Suspended => {
                pixels = None;
                window = None;
            }
            Event::RedrawRequested(_) => {
                if let (Some(pixels), Some(window)) = (&mut pixels, &window) {
                    world.draw(pixels.frame_mut());
                    pixels.render().unwrap();
                    window.request_redraw();
                }
            }
            _ => {}
        }
        if window.is_some() {
            world.update();
        }
    });
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
    let event_loop = EventLoopBuilder::new().build();
    log::info!("Hello from desktop!");
    _main(event_loop);
}
