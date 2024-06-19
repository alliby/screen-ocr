use crate::action::Action;
use crate::state::{AppState, Page, WindowState};
use femtovg::renderer::OpenGl;
use femtovg::{Align, Baseline, Canvas, Color, FillRule, FontId, Paint, Path, Solidity};
use std::f32::consts::PI;

const BACKGROUND_COLOR: Color = Color::rgbaf(0.01, 0.01, 0.01, 0.4);

pub fn app(canvas: &mut Canvas<OpenGl>, app: &AppState, window: &WindowState) -> Action {
    let (w, h) = (window.width, window.height);
    match app.current_page {
        Page::AreaSelect(_) => match app.selected_corners {
            (Some((x0, y0)), None) => {
                let (mouse_x, mouse_y) = window.cursor_position;
                draw_selection(canvas, w, h, x0, y0, mouse_x, mouse_y);
                return Action::Redraw;
            }
            (Some((x0, y0)), Some((x1, y1))) => {
                draw_selection(canvas, w, h, x0, y0, x1, y1);
                draw_button(
                    canvas,
                    &app.fonts,
                    "Cancel",
                    (x1.max(x0) - 110.0, y1.max(y0) + 10.0, 100.0, 50.0),
                );
                draw_button(
                    canvas,
                    &app.fonts,
                    "Ok",
                    (x1.max(x0) - 220.0, y1.max(y0) + 10.0, 100.0, 50.0),
                );
            }
            _ => {
                draw_button(
                    canvas,
                    &app.fonts,
                    "Exit",
                    ((w - 200.0) / 2.0, (h + 100.0) / 2.0, 200.0, 100.0),
                );
                draw_background(canvas, w, h);
            }
        },
    }
    Action::None
}

fn draw_background(canvas: &mut Canvas<OpenGl>, w: f32, h: f32) {
    let paint = Paint::color(BACKGROUND_COLOR);
    let mut path = Path::new();
    path.rect(0.0, 0.0, w, h);
    canvas.fill_path(&path, &paint);
}

fn draw_selection(canvas: &mut Canvas<OpenGl>, w: f32, h: f32, x0: f32, y0: f32, x1: f32, y1: f32) {
    let stroke_paint = Paint::color(Color::white());
    let fill_paint = Paint::color(BACKGROUND_COLOR).with_fill_rule(FillRule::EvenOdd);
    let mut path = Path::new();
    let mut stroke_path = Path::new();
    path.rect(0.0, 0.0, w, h);
    path.close();
    path.rect(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
    stroke_path.rect(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
    canvas.stroke_path(&stroke_path, &stroke_paint);
    canvas.fill_path(&path, &fill_paint);
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

fn draw_button(
    canvas: &mut Canvas<OpenGl>,
    fonts: &[FontId],
    text: &str,
    (x, y, w, h): (f32, f32, f32, f32),
) {
    let bg = Paint::color(Color::black());
    let mut path = Path::new();
    path.rounded_rect(x, y, w, h, 5.0);
    canvas.fill_path(&path, &bg);
    canvas.stroke_path(
        &path,
        &Paint::color(Color::rgb(200, 200, 200)).with_line_width(2.0),
    );
    let mut paint = Paint::color(Color::white());
    paint.set_text_align(Align::Center);
    paint.set_text_baseline(Baseline::Middle);
    paint.set_font_size(24.0);
    paint.set_font(&fonts);
    let _ = canvas.fill_text(x + w * 0.5, y + h * 0.5, text, &paint);
}
