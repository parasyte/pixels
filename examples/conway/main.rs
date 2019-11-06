use log::debug;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 400;
const SCREEN_HEIGHT: u32 = 300;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, surface, mut p_width, mut p_height, mut hidpi_factor) =
        create_window("Conway's Game of Life", &event_loop);

    let surface_texture = SurfaceTexture::new(p_width, p_height, surface);

    let mut life = ConwayGrid::new_random(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize);
    let mut pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?;
    let mut paused = false;

    let mut draw_state: Option<bool> = None;

    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            life.draw(pixels.get_frame());
            pixels.render();
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::P) {
                paused = !paused;
            }
            if input.key_pressed(VirtualKeyCode::Space) {
                // Space is frame-step, so ensure we're paused
                paused = true;
            }
            if input.key_pressed(VirtualKeyCode::R) {
                life.randomize();
            }
            // Handle mouse. This is a bit involved since support some simple
            // line drawing (mostly because it makes nice looking patterns).
            let (mouse_cell, mouse_prev_cell) = input
                .mouse()
                .map(|(mx, my)| {
                    let (dx, dy) = input.mouse_diff();
                    let prev_x = mx - dx;
                    let prev_y = my - dy;
                    let dpx = hidpi_factor as f32;
                    let (w, h) = (p_width as f32 / dpx, p_height as f32 / dpx);
                    let mx_i = ((mx / w) * (SCREEN_WIDTH as f32)).round() as isize;
                    let my_i = ((my / h) * (SCREEN_HEIGHT as f32)).round() as isize;
                    let px_i = ((prev_x / w) * (SCREEN_WIDTH as f32)).round() as isize;
                    let py_i = ((prev_y / h) * (SCREEN_HEIGHT as f32)).round() as isize;
                    ((mx_i, my_i), (px_i, py_i))
                })
                .unwrap_or_default();

            if input.mouse_pressed(0) {
                debug!("Mouse click at {:?}", mouse_cell);
                draw_state = Some(life.toggle(mouse_cell.0, mouse_cell.1));
            } else if let Some(draw_alive) = draw_state {
                let release = input.mouse_released(0);
                let held = input.mouse_held(0);
                debug!("Draw at {:?} => {:?}", mouse_prev_cell, mouse_cell);
                debug!("Mouse held {:?}, release {:?}", held, release);
                // If they either released (finishing the drawing) or are still
                // in the middle of drawing, keep going.
                if release || held {
                    debug!("Draw line of {:?}", draw_alive);
                    life.set_line(
                        mouse_prev_cell.0,
                        mouse_prev_cell.1,
                        mouse_cell.0,
                        mouse_cell.1,
                        draw_alive,
                    );
                }
                // If they let go or are otherwise not clicking anymore, stop drawing.
                if release || !held {
                    debug!("Draw end");
                    draw_state = None;
                }
            }
            // Adjust high DPI factor
            if let Some(factor) = input.hidpi_changed() {
                hidpi_factor = factor;
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                let size = size.to_physical(hidpi_factor);
                let width = size.width.round() as u32;
                let height = size.height.round() as u32;
                p_width = size.width.round() as u32;
                p_height = size.height.round() as u32;

                pixels.resize(width, height);
            }
            if !paused || input.key_pressed(VirtualKeyCode::Space) {
                life.update();
            }
            window.request_redraw();
        }
    });
}

// COPYPASTE: ideally this could be shared.

/// Create a window for the game.
///
/// Automatically scales the window to cover about 2/3 of the monitor height.
///
/// # Returns
///
/// Tuple of `(window, surface, width, height, hidpi_factor)`
/// `width` and `height` are in `PhysicalSize` units.
fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, pixels::wgpu::Surface, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    let hidpi_factor = window.hidpi_factor();

    // Get dimensions
    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;
    let (monitor_width, monitor_height) = {
        let size = window.current_monitor().size();
        (size.width / hidpi_factor, size.height / hidpi_factor)
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round();

    // Resize, center, and display the window
    let min_size = PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let surface = pixels::wgpu::Surface::create(&window);
    let size = default_size.to_physical(hidpi_factor);

    (
        window,
        surface,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
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

const BIRTH_RULE: [bool; 9] = [false, false, false, true, false, false, false, false, false];
const SURVIVE_RULE: [bool; 9] = [false, false, true, true, false, false, false, false, false];
const INITIAL_FILL: f32 = 0.3;

#[derive(Clone, Copy, Debug, Default)]
struct Cell {
    alive: bool,
    // Used for the trail effect. Always 255 if `self.alive` is true (We could
    // use an enum for Cell, but it makes several functions slightly more
    // complex, and doesn't actually make anything any simpler here, or save any
    // memory, so we don't)
    heat: u8,
}

impl Cell {
    fn new(alive: bool) -> Self {
        Self { alive, heat: 0 }
    }

    #[must_use]
    fn update_neibs(self, n: usize) -> Self {
        let next_alive = if self.alive {
            SURVIVE_RULE[n]
        } else {
            BIRTH_RULE[n]
        };
        self.next_state(next_alive)
    }

    #[must_use]
    fn next_state(mut self, alive: bool) -> Self {
        self.alive = alive;
        if self.alive {
            self.heat = 255;
        } else {
            self.heat = self.heat.saturating_sub(1);
        }
        self
    }

    fn set_alive(&mut self, alive: bool) {
        *self = self.next_state(alive);
    }

    fn cool_off(&mut self, decay: f32) {
        if !self.alive {
            let heat = (self.heat as f32 * decay).min(255.0).max(0.0);
            assert!(heat.is_finite());
            self.heat = heat as u8;
        }
    }
}

#[derive(Clone, Debug)]
struct ConwayGrid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
    // Should always be the same size as `cells`. When updating, we read from
    // `cells` and write to `scratch_cells`, then swap. Otherwise it's not in
    // use, and `cells` should be updated directly.
    scratch_cells: Vec<Cell>,
}

impl ConwayGrid {
    fn new_empty(width: usize, height: usize) -> Self {
        assert!(width != 0 && height != 0);
        let size = width.checked_mul(height).expect("too big");
        Self {
            cells: vec![Cell::default(); size],
            scratch_cells: vec![Cell::default(); size],
            width,
            height,
        }
    }

    fn new_random(width: usize, height: usize) -> Self {
        let mut result = Self::new_empty(width, height);
        result.randomize();
        result
    }

    fn randomize(&mut self) {
        let mut rng: randomize::PCG32 = generate_seed().into();
        for c in self.cells.iter_mut() {
            let alive = randomize::f32_half_open_right(rng.next_u32()) > INITIAL_FILL;
            *c = Cell::new(alive);
        }
        // run a few simulation iterations for aesthetics (If we don't, the
        // noise is ugly)
        for _ in 0..3 {
            self.update();
        }
        // Smooth out noise in the heatmap that would remain for a while
        for c in self.cells.iter_mut() {
            c.cool_off(0.4);
        }
    }

    fn count_neibs(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        let wi = self.width as isize;
        let hi = self.height as isize;
        for j in -1isize..=1 {
            for i in -1isize..=1 {
                if i == 0 && j == 0 {
                    continue;
                }
                // wrap around
                let xx = (x as isize + i + wi) % wi;
                let yy = (y as isize + j + hi) % hi;
                assert!(xx >= 0 && yy >= 0 && xx < wi && yy < hi);
                let i = self.grid_idx(xx, yy).expect("wrap-around bug?");
                if self.cells[i].alive {
                    count += 1;
                }
            }
        }
        count
    }

    fn update(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let neibs = self.count_neibs(x, y);
                let idx = x + y * self.width;
                let next = self.cells[idx].update_neibs(neibs);
                // Write into scratch_cells, since we're still reading from `self.cells`
                self.scratch_cells[idx] = next;
            }
        }
        std::mem::swap(&mut self.scratch_cells, &mut self.cells);
    }

    fn toggle(&mut self, x: isize, y: isize) -> bool {
        if let Some(i) = self.grid_idx(x, y) {
            let was_alive = self.cells[i].alive;
            self.cells[i].set_alive(!was_alive);
            !was_alive
        } else {
            false
        }
    }

    fn draw(&self, screen: &mut [u8]) {
        debug_assert_eq!(screen.len(), 4 * self.cells.len());
        for (c, pix) in self.cells.iter().zip(screen.chunks_exact_mut(4)) {
            let color = if c.alive {
                [0, 0xff, 0xff, 0xff]
            } else {
                [0, 0, c.heat, 0xff]
            };
            pix.copy_from_slice(&color);
        }
    }

    fn set_line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, alive: bool) {
        // probably should do sutherland-hodgeman if this were more serious.
        // instead just clamp the start pos, and draw until moving towards the
        // end pos takes us out of bounds.
        let x0 = x0.max(0).min(self.width as isize);
        let y0 = y0.max(0).min(self.height as isize);
        bresenham(x0, y0, x1, y1, |x, y| {
            if let Some(i) = self.grid_idx(x, y) {
                self.cells[i].set_alive(alive);
                true
            } else {
                false
            }
        });
    }

    fn grid_idx<I: std::convert::TryInto<usize>>(&self, x: I, y: I) -> Option<usize> {
        if let (Ok(x), Ok(y)) = (x.try_into(), y.try_into()) {
            Some(x + y * self.width)
        } else {
            None
        }
    }
}

fn bresenham(x0: isize, y0: isize, x1: isize, y1: isize, mut cb: impl FnMut(isize, isize) -> bool) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    // note: not quite signum, as signum(0) == 0
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    // bail out if cb returns false, otherwise keep going until x and y reach
    // the destination
    while cb(x, y) && (x != x1 || y != y1) {
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}
