use femtovg::FontId;
use std::time::Instant;
use winit::window::{CursorIcon, Window};

type Point = (f32, f32);

#[derive(Debug, Clone, Copy)]
pub enum Page {
    // Area Select Page with boolean to indicate if the drag is start or not
    AreaSelect(bool),
}

#[derive(Debug)]
pub struct AppState {
    pub window: Window,
    pub start_time: Instant,
    pub fonts: Vec<FontId>,
    pub cursor_position: Point,
    pub last_press: Option<Point>,
    pub last_release: Option<Point>,
    pub current_page: Page,
    pub selected_corners: (Option<Point>, Option<Point>),
    pub cursor_icon: CursorIcon,
}

impl AppState {
    pub fn new(window: Window) -> Self {
        let size = window.inner_size();
        let position = (size.width as f32 / 2.0, size.height as f32 / 2.0);
        window.set_cursor(CursorIcon::Crosshair);
        AppState {
            window,
            start_time: Instant::now(),
            fonts: Vec::new(),
            cursor_position: position,
            last_press: None,
            last_release: None,
            current_page: Page::AreaSelect(false),
            selected_corners: (None, None),
            cursor_icon: CursorIcon::Crosshair,
        }
    }
}

pub fn handle_state(state: &mut AppState) {
    match state.current_page {
        Page::AreaSelect(false) => {
            if let Some(cursor_position) = state.last_press {
                state.current_page = Page::AreaSelect(true);
                state.selected_corners.0 = Some(cursor_position);
            }
            state.window.request_redraw();
        }
        Page::AreaSelect(true) => {
            if let Some(cursor_position) = state.last_release {
                state.current_page = Page::AreaSelect(false);
                state.selected_corners.1 = Some(cursor_position);
                state.window.request_redraw();
            }
        }
    }
}
