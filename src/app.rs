use femtovg::FontId;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct AppState {
    pub start: Instant,
    pub width: u32,
    pub height: u32,
    pub fonts: Vec<FontId>,
    pub start_sel: bool,
    pub sel_corners: (Option<(f64, f64)>, Option<(f64, f64)>),
}
