use crate::action::Action;
use femtovg::FontId;
use std::time::Instant;
use winit::window::CursorIcon;
use winit::event::*;

type Point = (f32, f32);

#[derive(Debug, Clone)]
pub struct WindowState {
    pub cursor_icon: CursorIcon,
    pub cursor_position: Point,
    pub width: f32,
    pub height: f32,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            cursor_icon: CursorIcon::Crosshair,
            cursor_position: (0.0, 0.0),
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Page {
    // Area Select Page with boolean to indicate if the drag is start or not
    AreaSelect(bool),
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub start_time: Instant,
    pub fonts: Vec<FontId>,
    pub current_page: Page,
    pub selected_corners: (Option<Point>, Option<Point>),
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            start_time: Instant::now(),
            fonts: Vec::new(),
            current_page: Page::AreaSelect(false),
            selected_corners: (None, None),
        }
    }
}

pub fn handle_mouse_input(input: ElementState, window: &WindowState, app: &mut AppState) -> Action {
    if let ElementState::Pressed = input {
        match app.current_page {
            Page::AreaSelect(false) => {
                app.current_page = Page::AreaSelect(true);
                app.selected_corners = (Some(window.cursor_position), None);
                return Action::Redraw;
            }
            _ => {}
        }
    } else {
        match app.current_page {
            Page::AreaSelect(true) => {
                app.current_page = Page::AreaSelect(false);
                app.selected_corners.1 = Some(window.cursor_position);
            }
            _ => {}
        }
    }
    Action::None
}
