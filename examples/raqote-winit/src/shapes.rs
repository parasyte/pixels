use euclid::{Angle, Vector2D};
use raqote::*;

/// A context that can draw vector shapes
pub struct Shapes {
    // The main draw target
    dt: DrawTarget,

    // Center
    cx: f32,
    cy: f32,

    // Background color
    r: f32,
    g: f32,
    b: f32,

    // Two paths rotate in opposite directions
    t1: Transform,
    t2: Transform,
}

impl Shapes {
    /// Create a new Shapes
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            dt: DrawTarget::new(width as i32, height as i32),
            cx: (width / 2) as f32,
            cy: (width / 2) as f32,
            r: 0.0,
            g: 0.34,
            b: 0.88,
            t1: Transform::identity(),
            t2: Transform::identity(),
        }
    }

    /// Gain access to the underlying pixels
    pub fn frame(&self) -> &[u32] {
        self.dt.get_data()
    }

    /// Draw all of the shapes
    pub fn draw(&mut self, delta: f32) {
        // This is not a good way to blend colors.
        // But it's fine for a demo.
        self.r += delta;
        self.g += delta * 0.71;
        self.b += delta * 1.33;

        self.dt.clear(SolidSource {
            g: ((self.g.sin() + 1.0) * 127.5).round() as u8,
            r: ((self.r.sin() + 1.0) * 127.5).round() as u8,
            b: ((self.b.sin() + 1.0) * 127.5).round() as u8,
            a: 0xff,
        });

        let translate = Vector2D::new(-self.cx, -self.cy).to_transform();
        let inv_translate = translate
            .inverse()
            .unwrap_or_else(|| Vector2D::new(self.cx, self.cy).to_transform());

        self.t1 = self.t1.then(&translate);
        self.t1 = self.t1.then_rotate(Angle::radians(delta));
        self.t1 = self.t1.then(&inv_translate);
        self.dt.set_transform(&self.t1);

        let mut pb = PathBuilder::new();
        pb.move_to(100., 10.);
        pb.cubic_to(150., 40., 175., 0., 200., 10.);
        pb.quad_to(120., 100., 80., 200.);
        pb.quad_to(150., 180., 300., 300.);
        pb.close();
        let path = pb.finish();

        let gradient = Source::new_radial_gradient(
            Gradient {
                stops: vec![
                    GradientStop {
                        position: 0.2,
                        color: Color::new(0xff, 0, 0xff, 0),
                    },
                    GradientStop {
                        position: 0.8,
                        color: Color::new(0xff, 0xff, 0xff, 0xff),
                    },
                    GradientStop {
                        position: 1.,
                        color: Color::new(0xff, 0xff, 0, 0xff),
                    },
                ],
            },
            Point::new(150., 150.),
            128.,
            Spread::Pad,
        );
        self.dt.fill(&path, &gradient, &DrawOptions::new());

        self.t2 = self.t2.then(&translate);
        self.t2 = self.t2.then_rotate(Angle::radians(-delta));
        self.t2 = self.t2.then(&inv_translate);
        self.dt.set_transform(&self.t2);

        let mut pb = PathBuilder::new();
        pb.move_to(100., 100.);
        pb.line_to(300., 300.);
        pb.line_to(200., 300.);
        let path = pb.finish();

        self.dt.stroke(
            &path,
            &Source::Solid(SolidSource {
                r: 0x0,
                g: 0x0,
                b: 0x80,
                a: 0x80,
            }),
            &StrokeStyle {
                cap: LineCap::Round,
                join: LineJoin::Round,
                width: 10.,
                miter_limit: 2.,
                dash_array: vec![10., 18.],
                dash_offset: 16.,
            },
            &DrawOptions::new(),
        );
    }
}
