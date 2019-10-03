use pixels::{Error, Pixels, SurfaceTexture};
use winit::event;
use winit::event_loop::{ControlFlow, EventLoop};

fn scale_pixel_ferris(width: u32, height: u32) -> Vec<u8> {
    let mut px = Vec::new();

    const FERRIS_WIDTH: u32 = 11;
    const FERRIS_HEIGHT: u32 = 5;
    #[rustfmt::skip]
    const FERRIS: [u8; (FERRIS_WIDTH * FERRIS_HEIGHT) as usize] = [
        0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0,
        1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1,
        0, 0, 1, 1, 2, 1, 2, 1, 1, 0, 0,
        0, 1, 3, 1, 1, 2, 1, 1, 3, 1, 0,
        0, 0, 1, 3, 0, 0, 0, 3, 1, 0, 0,
    ];

    let scale = width / FERRIS_WIDTH;
    let top = (height - scale * FERRIS_HEIGHT) / 2;
    let bottom = height - top - 1;

    for y in 0..height {
        for x in 0..width {
            let rgba = if y < top || y >= bottom || x / scale >= FERRIS_WIDTH {
                [0xdd, 0xba, 0xdc, 0xff]
            } else {
                let i = x / scale + (y - top) / scale * FERRIS_WIDTH;

                match FERRIS[i as usize] {
                    0 => [0xdd, 0xba, 0xdc, 0xff],
                    1 => [0xf7, 0x4c, 0x00, 0xff],
                    2 => [0x00, 0x00, 0x00, 0xff],
                    3 => [0xa5, 0x2b, 0x00, 0xff],
                    _ => unreachable!(),
                }
            };

            px.extend_from_slice(&rgba);
        }
    }

    px
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();

    let (window, surface, width, height) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        let surface = wgpu::Surface::create(&window);
        let size = window.inner_size().to_physical(window.hidpi_factor());

        (window, surface, size.width as u32, size.height as u32)
    };

    let surface_texture = SurfaceTexture::new(width, height, &surface);
    let mut fb = Pixels::new(320, 240, surface_texture)?;

    fb.update(&scale_pixel_ferris(320, 240));

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::WindowEvent { event, .. } => match event {
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
            event::WindowEvent::RedrawRequested => fb.render(),
            _ => (),
        },
        event::Event::EventsCleared => window.request_redraw(),
        _ => (),
    });
}
