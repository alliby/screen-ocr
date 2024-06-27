use crate::state::*;

use femtovg::renderer::OpenGl;
use femtovg::{Align, Baseline, Canvas, Color, FillRule, FontId, Paint, Path, Solidity};
use std::f32::consts::PI;
use winit::window::CursorIcon;

const BACKGROUND_COLOR: Color = Color::rgbaf(0.01, 0.01, 0.01, 0.4);

pub fn app(canvas: &mut Canvas<OpenGl>, app_state: &mut AppState) {
    match app_state.action {
        Action::AreaSelect(mut state) => draw_area_select(canvas, app_state, &mut state),
        Action::AreaConfirm(mut state) => draw_area_confirm(canvas, app_state, &mut state),
        Action::StartScreenshot => {
            let (w, h) = (app_state.width, app_state.height);
            let t = app_state.start_time.elapsed().as_secs_f32();
            draw_background(canvas, w, h);
            draw_spinner(canvas, w / 2.0, h / 2.0, 30.0, t, Color::white());
        }
    }
}

fn is_cursor_in(cursor_position: (f32, f32), rectangle: (f32, f32, f32, f32)) -> bool {
    cursor_position.0 > rectangle.0
        && cursor_position.1 > rectangle.1
        && cursor_position.0 < rectangle.0 + rectangle.2
        && cursor_position.1 < rectangle.1 + rectangle.3
}

fn draw_area_select(
    canvas: &mut Canvas<OpenGl>,
    app_state: &mut AppState,
    state: &mut AreaSelectState,
) {
    let (w, h) = (app_state.width, app_state.height);
    if let Some((x0, y0)) = state.start_point {
        let (mouse_x, mouse_y) = app_state.cursor_position;
        draw_selection(canvas, w, h, x0, y0, mouse_x, mouse_y);
    } else {
        draw_background(canvas, w, h);
    }
}

fn draw_area_confirm(
    canvas: &mut Canvas<OpenGl>,
    app_state: &mut AppState,
    state: &mut AreaConfirmState,
) {
    let (w, h) = (app_state.width, app_state.height);
    let ((x0, y0), (x1, y1)) = state.corners;
    let mut cursor_icon = app_state.cursor_icon;
    let mouse_click = app_state.last_press.is_some();
    let ok_rect = (x1.max(x0) - 220.0, y1.max(y0) + 10.0, 100.0, 50.0);
    let close_rect = (w - 75.0, 30.0, 50.0, 50.0);
    let ok_btn_over = is_cursor_in(app_state.cursor_position, ok_rect);
    let close_btn_over = is_cursor_in(app_state.cursor_position, close_rect);
    let mut close_btn_color = Color::rgb(46, 46, 46);
    let mut ok_btn_color = Color::rgb(16, 16, 16);

    if ok_btn_over {
        ok_btn_color.set_alphaf(0.7);
        cursor_icon = CursorIcon::Pointer;
    }
    if close_btn_over {
        close_btn_color = Color::rgba(200, 16, 16, 220);
        cursor_icon = CursorIcon::Pointer;
    }

    draw_selection(canvas, w, h, x0, y0, x1, y1);
    draw_button(canvas, &app_state.fonts, "Ok", ok_rect, ok_btn_color);
    draw_close_btn(canvas, close_rect, close_btn_color);

    if ok_btn_over && mouse_click {
        app_state.action = Action::AreaConfirm(AreaConfirmState {
            confirm: true,
            corners: state.corners,
        });
    } else if close_btn_over && mouse_click {
        app_state.should_exit = true;
    } else if mouse_click {
        app_state.action = Action::AreaSelect(AreaSelectState {
            start_point: app_state.last_press,
        });
    }
    app_state.windows[app_state.active_window].set_cursor(cursor_icon);
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
