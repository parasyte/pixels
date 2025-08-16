#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

impl World {
    fn new() -> Self {
        Self { box_x: 24, box_y: 16, velocity_x: 1, velocity_y: 1 }
    }
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
    fn draw(&self, frame: &mut [u8]) {
        for (i, px) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;
            let inside = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;
            let rgba = if inside { [0x5e, 0x48, 0xe8, 0xff] } else { [0x48, 0xb2, 0xe8, 0xff] };
            px.copy_from_slice(&rgba);
        }
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

struct App {
    window: Option<&'static Window>,     // leaked &'static Window
    pixels: Option<Pixels<'static>>,     // borrows the window
    world: World,
}

impl App {
    fn new() -> Self {
        Self { window: None, pixels: None, world: World::new() }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, elwt: &ActiveEventLoop) {
        if self.window.is_none() {
            // Create window
            let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
            let attrs = WindowAttributes::default()
                .with_title("Hello Pixels")
                .with_inner_size(size)
                .with_min_inner_size(size);
            let window = elwt.create_window(attrs).expect("create window");

            // Leak to get &'static Window so Pixels can live in the struct
            let window: &'static Window = Box::leak(Box::new(window));
            let win_size = window.inner_size();

            // Create Pixels
            let surface = SurfaceTexture::new(win_size.width, win_size.height, window);
            let pixels = Pixels::new(WIDTH, HEIGHT, surface).expect("create pixels");

            self.window = Some(window);
            self.pixels = Some(pixels);
        }
    }

    fn window_event(
        &mut self,
        elwt: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window else { return };
        if window_id != window.id() { return; }

        match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => elwt.exit(),
            WindowEvent::Resized(size) => {
                if let Some(pixels) = self.pixels.as_mut() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);
                        elwt.exit();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = self.pixels.as_mut() {
                    self.world.draw(pixels.frame_mut());
                    if let Err(err) = pixels.render() {
                        log_error("pixels.render", err);
                        elwt.exit();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _elwt: &ActiveEventLoop) {
        if let Some(window) = self.window {
            self.world.update();
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new().expect("create event loop");
    let mut app = App::new();
    let res = event_loop.run_app(&mut app);
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}
