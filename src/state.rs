use std::time::Instant;
use std::sync::Arc;
use vello::kurbo::{Point, Rect};
use vello::peniko::Blob;
use vello::Scene;
use winit::window::CursorIcon;

// elements indices

// Area Select elements
pub const FULL_SCREEN_OVERLAY: usize = 0;
pub const SELECTED_RECT: usize = 1;
pub const CONFIRM_BTN: usize = 2;
pub const TOP_LEFT_BTN: usize = 3;
pub const TOP_RIGHT_BTN: usize = 4;
pub const BOTTOM_RIGHT_BTN: usize = 5;
pub const BOTTOM_LEFT_BTN: usize = 6;

#[derive(Default, Debug, Clone, Copy)]
pub struct ViewElement {
    pub bound: Rect,
    pub active: bool,
    pub cursor: CursorIcon,
    pub mouse_enter: bool,
    pub mouse_press: bool,
}

#[derive(Default, Clone)]
pub struct View {
    pub mouse_position: Point,
    pub scene: Scene,
    pub elems: Vec<ViewElement>,
}

#[derive(Default, Clone)]
pub struct AppState {
    pub page: Page,
    pub page_data: PageData,
    pub should_exit: bool,
    pub redraw: bool,
    pub damaged: bool,
    pub switch_windows: bool,
    pub screen_width: f64,
    pub screen_height: f64,
}

#[derive(Default, Copy, Clone)]
pub enum Page {
    #[default]
    AreaSelect,
    TextExtract,
}

#[derive(Debug, Clone)]
pub enum PageData {
    AreaSelect {
        grab: Option<Point>,
        resize: Option<usize>,
        rect: Rect,
    },
    TextExtract {
	rect: Rect,
        time: Instant,
	window_created: bool,
	screen_captured: bool,
	blob: Blob<u8>
    },
}

impl Default for PageData {
    fn default() -> Self {
        Self::AreaSelect {
            grab: None,
            resize: None,
            rect: Rect::ZERO,
        }
    }
}

impl AppState {
    pub fn callbacks(&self) -> Vec<fn(&mut AppState, &mut View)> {
        match self.page {
            Page::AreaSelect => vec![
                // the full screen overlay
                |state, view| {
                    let mouse = view.mouse_position;
                    let PageData::AreaSelect { ref mut rect, .. } = state.page_data else {
                        return;
                    };
                    // if no press the second call is for the mouse released
                    let mouse_press = view.elems[FULL_SCREEN_OVERLAY].mouse_press;
                    if mouse_press {
                        view.elems[SELECTED_RECT].bound.x0 = mouse.x;
                        view.elems[SELECTED_RECT].bound.y0 = mouse.y;
                        view.elems[CONFIRM_BTN].bound = Rect::ZERO;
                        for i in SELECTED_RECT..=BOTTOM_LEFT_BTN {
                            view.elems[i].active = false;
                        }
                    } else {
                        view.elems[CONFIRM_BTN].active = true;
                        view.elems[SELECTED_RECT].active = true;
                        *rect = view.elems[SELECTED_RECT].bound;
                        for i in SELECTED_RECT..=BOTTOM_LEFT_BTN {
                            view.elems[i].active = true;
                        }
                    }
                },
                // for the selected rect
                |state, view| {
                    let mouse = view.mouse_position;
                    let PageData::AreaSelect {
                        ref mut grab,
                        ref mut rect,
                        ..
                    } = state.page_data
                    else {
                        return;
                    };

                    if grab.is_none() {
                        *grab = Some(mouse);
                    } else {
                        *grab = None;
                        *rect = view.elems[SELECTED_RECT].bound;
                    }
                },

		// for the confirm button
                |state, _| {
                    let PageData::AreaSelect { rect, .. } = state.page_data else {
                        return;
                    };
                    state.damaged = true;
		    state.switch_windows = true;
                    state.page = Page::TextExtract;
                    state.page_data = PageData::TextExtract {
			rect,
			time: Instant::now(),
			blob: Blob::new(Arc::new([])),
			window_created: false,
			screen_captured: false,
                    };
                },

                // Resize Buttons Callbacks
                |state, view| {
                    let PageData::AreaSelect {
                        ref mut resize,
                        ref mut rect,
                        ..
                    } = state.page_data
                    else {
                        return;
                    };

                    if resize.is_none() {
                        *resize = Some(TOP_LEFT_BTN);
                    } else {
                        *resize = None;
                        *rect = view.elems[SELECTED_RECT].bound;
                    }
                },
                |state, view| {
                    let PageData::AreaSelect {
                        ref mut resize,
                        ref mut rect,
                        ..
                    } = state.page_data
                    else {
                        return;
                    };

                    if resize.is_none() {
                        *resize = Some(TOP_RIGHT_BTN);
                    } else {
                        *resize = None;
                        *rect = view.elems[SELECTED_RECT].bound;
                    }
                },
                |state, view| {
                    let PageData::AreaSelect {
                        ref mut resize,
                        ref mut rect,
                        ..
                    } = state.page_data
                    else {
                        return;
                    };

                    if resize.is_none() {
                        *resize = Some(BOTTOM_RIGHT_BTN);
                    } else {
                        *resize = None;
                        *rect = view.elems[SELECTED_RECT].bound;
                    }
                },
                |state, view| {
                    let PageData::AreaSelect {
                        ref mut resize,
                        ref mut rect,
                        ..
                    } = state.page_data
                    else {
                        return;
                    };

                    if resize.is_none() {
                        *resize = Some(BOTTOM_LEFT_BTN);
                    } else {
                        *resize = None;
                        *rect = view.elems[SELECTED_RECT].bound;
                    }
                },
            ],

            Page::TextExtract => vec![],
        }
    }

    pub fn view_elements(&self) -> Vec<ViewElement> {
        match self.page {
            Page::AreaSelect => vec![
                // full screen overlay
                ViewElement {
                    cursor: CursorIcon::Crosshair,
                    bound: Rect::new(0.0, 0.0, self.screen_width, self.screen_height),
                    active: true,
                    ..Default::default()
                },
                // Selected Rectangle
                ViewElement {
                    cursor: CursorIcon::Grab,
                    ..Default::default()
                },
                // Confirm Button
                ViewElement {
                    cursor: CursorIcon::Pointer,
                    ..Default::default()
                },
                // Top Left Button
                ViewElement {
                    cursor: CursorIcon::Pointer,
                    ..Default::default()
                },
                // Top Right Button
                ViewElement {
                    cursor: CursorIcon::Pointer,
                    ..Default::default()
                },
                // Bottom Right Button
                ViewElement {
                    cursor: CursorIcon::Pointer,
                    ..Default::default()
                },
                // Bottom Left Button
                ViewElement {
                    cursor: CursorIcon::Pointer,
                    ..Default::default()
                },
            ],

            Page::TextExtract => vec![],
        }
    }
}
