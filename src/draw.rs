use crate::app::App;
use crate::helper::Rectangle;
use crate::state::*;

use femtovg::renderer::OpenGl;
use femtovg::{Align, Baseline, Canvas, Color, FillRule, FontId, ImageId, Paint, Path, Solidity};
use std::f32::consts::PI;
use winit::window::CursorIcon;

const BACKGROUND_COLOR: Color = Color::rgbaf(0.01, 0.01, 0.01, 0.4);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ImageKey {
    Screenshot,
    BluredScreen,
}

pub fn app(app: &mut App) {
    match app.action {
        Action::AreaSelect(state) => draw_area_select(app, &state),
        Action::AreaConfirm(mut state) => draw_area_confirm(app, &mut state),
        Action::ExtractText(ExtractionState::Extracting) => {
            let canvas = &mut app.render[app.active].canvas;
	    let blur_img_id = app.img_ids[&ImageKey::BluredScreen];
            let (w, h) = (app.width, app.height);
            let t = app.start_time.elapsed().as_secs_f32();
            draw_image(canvas, blur_img_id, (0.0, 0.0, w, h));
            draw_spinner(canvas, w / 2.0, h / 2.0, 30.0, t, Color::white());
        }
        Action::ExtractText(ExtractionState::Extracted) => {
            let canvas = &mut app.render[app.active].canvas;
            let img_id = app.img_ids[&ImageKey::Screenshot];
            let (w, h) = (app.width, app.height);
            draw_image(canvas, img_id, (0.0, 0.0, w, h));
        }
        _ => {}
    }
}

fn is_cursor_in(cursor_position: (f32, f32), rect: impl Into<Rectangle>) -> bool {
    let rect = rect.into();
    cursor_position.0 > rect.x
        && cursor_position.1 > rect.y
        && cursor_position.0 < rect.x + rect.width
        && cursor_position.1 < rect.y + rect.height
}

fn draw_area_select(app: &mut App, state: &AreaSelectState) {
    let canvas = &mut app.render[app.active].canvas;
    let (w, h) = (app.width, app.height);
    let mut cursor_icon = app.cursor_icon;
    let close_rect = (w - 75.0, 30.0, 50.0, 50.0);
    let close_btn_over = is_cursor_in(app.cursor_position, close_rect);
    let mut close_btn_color = Color::rgb(46, 46, 46);
    if close_btn_over {
        close_btn_color = Color::rgba(200, 16, 16, 220);
        cursor_icon = CursorIcon::Pointer;
    }

    if let Some((x0, y0)) = state.start_point {
        if is_cursor_in((x0, y0), close_rect) {
            app.should_exit = true;
        }
        let (mouse_x, mouse_y) = app.cursor_position;
        draw_selection(canvas, w, h, (x0, y0), (mouse_x, mouse_y));
    } else {
        draw_background(canvas, w, h);
        draw_round_btn(canvas, close_rect, close_btn_color);
        app.windows[app.active].set_cursor(cursor_icon);
    }
}

fn draw_area_confirm(app: &mut App, state: &mut AreaConfirmState) {
    let canvas = &mut app.render[app.active].canvas;
    let (w, h) = (app.width, app.height);
    let ((x0, y0), (x1, y1)) = state.corners;
    let mut cursor_icon = app.cursor_icon;
    let mut reset_click = false;
    let mouse_click = app.last_press.is_some();
    let ok_rect = (x1.max(x0) - 220.0, y1.max(y0) + 10.0, 100.0, 50.0);
    let ok_btn_hover = is_cursor_in(app.cursor_position, ok_rect);
    let mut ok_btn_color = Color::rgb(16, 16, 16);

    if ok_btn_hover {
        ok_btn_color.set_alphaf(0.7);
        cursor_icon = CursorIcon::Pointer;
    }

    draw_selection(canvas, w, h, (x0, y0), (x1, y1));
    draw_button(canvas, &app.fonts, "Ok", ok_rect, ok_btn_color);

    let rect = Rectangle::from(state.corners);
    let combinations = [
        (CursorIcon::NwResize, (rect.x, rect.y)),
        (CursorIcon::SwResize, (rect.x, rect.y + rect.height)),
        (CursorIcon::NeResize, (rect.x + rect.width, rect.y)),
        (
            CursorIcon::SeResize,
            (rect.x + rect.width, rect.y + rect.height),
        ),
    ];
    for (icon, (x, y)) in combinations {
        let size = 20.0;
        let btn_rect = Rectangle::from_center_size(x, y, size);
        let btn_hhover = is_cursor_in(app.cursor_position, btn_rect);
        if btn_hhover {
            cursor_icon = icon;
        }
        if mouse_click && btn_hhover {
            let start_point = match icon {
                CursorIcon::NwResize => (x + rect.width, y + rect.height),
                CursorIcon::SwResize => (x + rect.width, y - rect.height),
                CursorIcon::NeResize => (x - rect.width, y + rect.height),
                _ => (x - rect.width, y - rect.height),
            };
            app.last_press = Some(start_point);
        }
        draw_round_btn(canvas, btn_rect, Color::white());
    }

    if ok_btn_hover && mouse_click {
        reset_click = true;
        app.action = Action::AreaConfirm(AreaConfirmState {
            confirm: true,
            corners: state.corners,
        });
    }

    if mouse_click && !reset_click {
        app.action = Action::AreaSelect(AreaSelectState {
            start_point: app.last_press,
        });
    }

    app.windows[app.active].set_cursor(cursor_icon);
}

fn draw_round_btn(canvas: &mut Canvas<OpenGl>, rect: impl Into<Rectangle>, color: Color) {
    let rect = rect.into();
    let mut path = Path::new();
    let paint = Paint::color(color);
    let cx = rect.x + rect.width * 0.5;
    let cy = rect.y + rect.height * 0.5;
    path.circle(cx, cy, rect.width * 0.5);
    canvas.fill_path(&path, &paint);
}

fn draw_background(canvas: &mut Canvas<OpenGl>, w: f32, h: f32) {
    let paint = Paint::color(BACKGROUND_COLOR);
    let mut path = Path::new();
    path.rect(0.0, 0.0, w, h);
    canvas.fill_path(&path, &paint);
}

fn draw_selection(canvas: &mut Canvas<OpenGl>, w: f32, h: f32, p1: (f32, f32), p2: (f32, f32)) {
    let stroke_paint = Paint::color(Color::white());
    let fill_paint = Paint::color(BACKGROUND_COLOR).with_fill_rule(FillRule::EvenOdd);
    let mut path = Path::new();
    let mut stroke_path = Path::new();
    path.rect(0.0, 0.0, w, h);
    path.close();
    let rect = Rectangle::from((p1, p2));
    path.rect(rect.x, rect.y, rect.width, rect.height);
    stroke_path.rect(rect.x, rect.y, rect.width, rect.height);
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
    rect: impl Into<Rectangle>,
    color: Color,
) {
    let rect = rect.into();
    let bg = Paint::color(color);
    let mut path = Path::new();
    path.rounded_rect(rect.x, rect.y, rect.width, rect.height, 5.0);
    canvas.fill_path(&path, &bg);
    let mut paint = Paint::color(Color::white());
    paint.set_text_align(Align::Center);
    paint.set_text_baseline(Baseline::Middle);
    paint.set_font_size(24.0);
    paint.set_font(fonts);
    let _ = canvas.fill_text(
        rect.x + (rect.width / 2.0),
        rect.y + (rect.height / 2.0),
        text,
        &paint,
    );
}

fn draw_image(canvas: &mut Canvas<OpenGl>, img_id: ImageId, rect: impl Into<Rectangle>) {
    let rect = rect.into();
    let mut path = Path::new();
    path.rect(rect.x, rect.y, rect.width, rect.height);
    let img = Paint::image(img_id, rect.x, rect.y, rect.width, rect.height, 0.0, 1.0);
    canvas.fill_path(&path, &img);
}
