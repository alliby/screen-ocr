use crate::state::{AppState, Page};
use femtovg::renderer::OpenGl;
use femtovg::{Align, Baseline, Canvas, Color, FillRule, FontId, Paint, Path, Solidity};
use std::f32::consts::PI;
use winit::window::CursorIcon;

const BACKGROUND_COLOR: Color = Color::rgbaf(0.01, 0.01, 0.01, 0.4);

pub fn app(canvas: &mut Canvas<OpenGl>, state: &mut AppState) {
    match state.current_page {
        Page::AreaSelect(_) => draw_area_select_page(canvas, state),
    }
}

fn is_cursor_in(cursor_position: (f32, f32), rectangle: (f32, f32, f32, f32)) -> bool {
    cursor_position.0 > rectangle.0
        && cursor_position.1 > rectangle.1
        && cursor_position.0 < rectangle.0 + rectangle.2
        && cursor_position.1 < rectangle.1 + rectangle.3
}

fn draw_area_select_page(canvas: &mut Canvas<OpenGl>, state: &mut AppState) {
    let size = state.window.inner_size();
    let (w, h) = (size.width as f32, size.height as f32);

    let close_btn_rect = (w - 60.0, 10.0, 50.0, 50.0);
    let hover = is_cursor_in(state.cursor_position, close_btn_rect);
    let mut close_btn_color = Color::rgb(46, 46, 46);

    if hover {
        close_btn_color = Color::rgba(200, 16, 16, 220);
        state.window.set_cursor(CursorIcon::Pointer);
    } else {
        state.window.set_cursor(state.cursor_icon);
    }

    draw_background(canvas, w, h);
    draw_close_btn(canvas, close_btn_rect, close_btn_color);

    if hover && state.last_press.is_some() {
        std::process::exit(0);
    }

    if let Some((x1, y1)) = state.last_release {
        let Some((x0, y0)) = state.selected_corners.0 else {
            return;
        };
        draw_selection(canvas, w, h, x0, y0, x1, y1);
        let ok_rect = (x1.max(x0) - 220.0, y1.max(y0) + 10.0, 100.0, 50.0);
        let exit_rect = (x1.max(x0) - 110.0, y1.max(y0) + 10.0, 100.0, 50.0);
        let mut btn_color = Color::rgb(16, 16, 16);
        if is_cursor_in(state.cursor_position, ok_rect)
            || is_cursor_in(state.cursor_position, exit_rect)
        {
            btn_color.set_alphaf(0.7);
            state.window.set_cursor(CursorIcon::Pointer);
        } else {
            state.window.set_cursor(state.cursor_icon);
        }
        draw_button(canvas, &state.fonts, "Cancel", ok_rect, btn_color);
        draw_button(canvas, &state.fonts, "Ok", exit_rect, btn_color);
    }

    if let Some((x, y)) = state.last_press {
        let (mouse_x, mouse_y) = state.cursor_position;
        draw_selection(canvas, w, h, x, y, mouse_x, mouse_y);
        state.window.request_redraw();
    }

}

fn draw_close_btn(canvas: &mut Canvas<OpenGl>, rect: (f32, f32, f32, f32), color: Color) {
    let mut path = Path::new();
    let paint = Paint::color(color);
    let (x, y, w, h) = rect;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    path.circle(cx, cy, w * 0.5);
    canvas.fill_path(&path, &paint);
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
    color: Color,
) {
    let bg = Paint::color(color);
    let mut path = Path::new();
    path.rounded_rect(x, y, w, h, 5.0);
    canvas.fill_path(&path, &bg);
    let mut paint = Paint::color(Color::white());
    paint.set_text_align(Align::Center);
    paint.set_text_baseline(Baseline::Middle);
    paint.set_font_size(24.0);
    paint.set_font(&fonts);
    let _ = canvas.fill_text(x + w / 2.0, y + h / 2.0, text, &paint);
}
