use tiny_skia::*;

pub fn draw(pixmap: &mut Pixmap, delta: f32) {
    let mut paint1 = Paint::default();
    paint1.set_color_rgba8(50, 107, 160, 255);
    paint1.anti_alias = true;

    let mut paint2 = Paint::default();
    paint2.set_color_rgba8(255, 125, 0, 150);
    paint2.anti_alias = true;

    let mut paint3 = Paint::default();
    paint3.set_color_rgba8(205, 205, 205, 205);
    paint3.anti_alias = true;

    let mut paint4 = Paint::default();
    paint4.set_color_rgba8(128, 0, 128, 255);
    paint4.anti_alias = true;

    let mut paint5 = Paint::default();
    paint5.set_color_rgba8(20, 205, 25, 205);
    paint5.anti_alias = true;

    let path1 = PathBuilder::from_circle(400.0, 400.0, 300.0).unwrap();

    let path2 = {
        let mut pb = PathBuilder::new();
        pb.move_to(940.0, 60.0);
        pb.line_to(840.0, 940.0);
        pb.cubic_to(620.0, 840.0, 340.0, 800.0, 60.0, 800.0);
        pb.cubic_to(260.0, 460.0, 560.0, 160.0, 940.0, 60.0);
        pb.close();
        pb.finish().unwrap()
    };

    let mut stroke = Stroke::default();
    pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
    pixmap.fill_path(
        &path1,
        &paint1,
        FillRule::Winding,
        Transform::from_rotate_at(delta * 15.0, 500.0, 500.0),
        None,
    );

    stroke.width = 4.0;
    pixmap.stroke_path(
        &path1,
        &paint5,
        &stroke,
        Transform::from_rotate_at(delta * 15.0, 500.0, 500.0),
        None,
    );

    stroke.width = 48.0;
    pixmap.stroke_path(
        &path1,
        &paint4,
        &stroke,
        Transform::from_rotate_at(-delta * 25.0, 500.0, 500.0).post_scale(0.75, 0.75),
        None,
    );

    pixmap.fill_path(
        &path2,
        &paint2,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    stroke.width = 8.0;
    pixmap.stroke_path(&path2, &paint3, &stroke, Transform::identity(), None);
}
