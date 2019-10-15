use std::env;
use std::time::Instant;

use pixels::{Error, Pixels, SurfaceTexture};
use simple_invaders::{Controls, Direction, World, SCREEN_HEIGHT, SCREEN_WIDTH};
use winit::event;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();

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
    let mut fb = Pixels::new(224, 256, surface_texture)?;
    let mut invaders = World::new(debug);
    let mut last = Instant::now();

    let mut controls = Controls::default();
    let mut last_state = false;
    let mut button_state = false;
    let mut rising_edge = false;

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::WindowEvent { event, .. } => match event {
            // Close events
            event::WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        virtual_keycode: Some(event::VirtualKeyCode::Escape),
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            }
            | event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

            // Keyboard controls
            event::WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        virtual_keycode: Some(virtual_code),
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            } => match virtual_code {
                event::VirtualKeyCode::Left => controls.direction = Direction::Left,
                event::VirtualKeyCode::Right => controls.direction = Direction::Right,
                event::VirtualKeyCode::Space => button_state = true,
                _ => (),
            },

            event::WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        virtual_keycode: Some(virtual_code),
                        state: event::ElementState::Released,
                        ..
                    },
                ..
            } => match virtual_code {
                event::VirtualKeyCode::Left => controls.direction = Direction::Still,
                event::VirtualKeyCode::Right => controls.direction = Direction::Still,
                event::VirtualKeyCode::Space => button_state = false,
                _ => (),
            },

            // Adjust high DPI factor
            event::WindowEvent::HiDpiFactorChanged(factor) => hidpi_factor = factor,

            // Resize the window
            event::WindowEvent::Resized(size) => {
                let size = size.to_physical(hidpi_factor);
                let width = size.width.round() as u32;
                let height = size.height.round() as u32;

                fb.resize(width, height);
            }

            // Redraw the screen
            event::WindowEvent::RedrawRequested => fb.render(invaders.draw()),

            _ => (),
        },
        event::Event::EventsCleared => {
            // Get a new delta time.
            let now = Instant::now();
            let dt = now.duration_since(last);
            last = now;

            // Compute rising edge based on current and last button states
            rising_edge = button_state && !last_state;
            last_state = button_state;

            // Fire button only uses rising edge
            controls.fire = rising_edge;

            // Update the game logic and request redraw
            invaders.update(&dt, &controls);
            window.request_redraw();
        }
        _ => (),
    });
}
