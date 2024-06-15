use crate::app::AppState;
use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color, FillRule, FontId, Paint, Path, Solidity};
use std::f32::consts::PI;

const BACKGROUND_COLOR: Color = Color::rgbaf(0.8, 0.8, 0.8, 0.6);

pub fn app(canvas: &mut Canvas<OpenGl>, app: &AppState) {
    let (w, h) = (app.width as f32, app.height as f32);
    if let (Some((x0, y0)), Some((x1, y1))) = app.sel_corners {
        draw_selection(canvas, w, h, x0 as _, y0 as _, x1 as _, y1 as _);
    } else {
        draw_background(canvas, w, h);
    }
    draw_paragraph(
        canvas,
        w / 2.0,
        h / 2.0,
        &app.fonts,
        40.0,
        "السلام عليكم",
        Color::white(),
    );
    draw_spinner(
        canvas,
        w / 2.0,
        h / 2.0,
        20.0,
        app.start.elapsed().as_secs_f32(),
        Color::white(),
    );
}

fn draw_background(canvas: &mut Canvas<OpenGl>, w: f32, h: f32) {
    let paint = Paint::color(BACKGROUND_COLOR);
    let mut path = Path::new();
    path.rect(0.0, 0.0, w, h);
    canvas.fill_path(&path, &paint);
}

fn draw_selection(canvas: &mut Canvas<OpenGl>, w: f32, h: f32, x0: f32, y0: f32, x1: f32, y1: f32) {
    let stroke_paint = Paint::color(Color::black());
    let fill_paint = Paint::color(BACKGROUND_COLOR).with_fill_rule(FillRule::EvenOdd);
    let mut path = Path::new();
    path.rect(0.0, 0.0, w, h);
    path.close();
    path.rect(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
    canvas.fill_path(&path, &fill_paint);
    canvas.stroke_path(&path, &stroke_paint);
}

fn draw_paragraph(
    canvas: &mut Canvas<OpenGl>,
    x: f32,
    y: f32,
    font: &[FontId],
    font_size: f32,
    text: &str,
    color: Color,
) {
    let mut paint = Paint::color(color);

    paint.set_font(font);
    paint.set_font_size(font_size);

    let font_metrics = canvas.measure_font(&paint).expect("Error measuring font");

    let width = canvas.width() as f32;
    let mut y = y;

    let lines = canvas
        .break_text_vec(width, text, &paint)
        .expect("Error while breaking text");

    for line_range in lines {
        if let Ok(_res) = canvas.fill_text(x, y, &text[line_range], &paint) {
            y += font_metrics.height();
        }
    }
}

fn draw_spinner(canvas: &mut Canvas<OpenGl>, cx: f32, cy: f32, r: f32, t: f32, color: Color) {
    let a0 = 0.0 + t * 6.0;
    let a1 = PI + t * 6.0;
    let r0 = r;
    let r1 = r * 0.9;

    canvas.save();

    let mut path = Path::new();
    path.arc(cx, cy, r0, a0, a1, Solidity::Hole);
    path.arc(cx, cy, r1, a1, a0, Solidity::Solid);
    path.close();

    let paint = Paint::color(color);
    canvas.fill_path(&path, &paint);

    canvas.restore();
}
