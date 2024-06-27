use crate::render;
use crate::render::RenderState;
use femtovg::FontId;
use std::time::Instant;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, Window};
use glutin::context::PossiblyCurrentGlContext;

type Point = (f32, f32);

#[derive(Debug, Clone, Copy)]
pub struct AreaSelectState {
    pub start_point: Option<Point>,
}

#[derive(Debug, Clone, Copy)]
pub struct AreaConfirmState {
    pub confirm: bool,
    pub corners: (Point, Point),
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    AreaSelect(AreaSelectState),
    AreaConfirm(AreaConfirmState),
    StartScreenshot,
}

#[derive(Debug)]
pub struct AppState {
    pub windows: Vec<Window>,
    pub active_window: usize,
    pub width: f32,
    pub height: f32,
    pub start_time: Instant,
    pub fonts: Vec<FontId>,
    pub cursor_position: Point,
    pub last_press: Option<Point>,
    pub last_release: Option<Point>,
    pub action: Action,
    pub selected_corners: Option<(Point, Point)>,
    pub cursor_icon: CursorIcon,
    pub should_exit: bool,
}

impl AppState {
    pub fn new(window: Window) -> Self {
        AppState {
            windows: vec![window],
	    active_window: 0,
            width: 800.0,
            height: 600.0,
            start_time: Instant::now(),
            fonts: Vec::new(),
            cursor_position: Default::default(),
            last_press: None,
            last_release: None,
            action: Action::AreaSelect(AreaSelectState { start_point: None }),
            selected_corners: None,
            cursor_icon: CursorIcon::Crosshair,
            should_exit: false,
        }
    }
}

pub fn handle_state(
    app_state: &mut AppState,
    render_state: &mut RenderState,
    event_loop: &ActiveEventLoop,
) {
    if app_state.should_exit {
        event_loop.exit();
        return;
    }
    match app_state.action {
        Action::AreaSelect(ref mut state) => {
            if app_state.last_press.is_some() {
                state.start_point = app_state.last_press;
                app_state.windows[app_state.active_window].request_redraw();
            }
            if let Some(cursor_position) = app_state.last_release {
                let Some(point_1) = state.start_point else {
                    return;
                };
                app_state.action = Action::AreaConfirm(AreaConfirmState {
                    confirm: false,
                    corners: (point_1, cursor_position),
                });
                app_state.windows[app_state.active_window].request_redraw();
            }
        }
        Action::AreaConfirm(AreaConfirmState {
            confirm: true,
            corners,
        }) => {
            let ((x0, y0), (x1, y1)) = corners;
            let (x, y, w, h) = (x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
            app_state.width = w;
            app_state.height = h;
            app_state.selected_corners = Some(corners);
            app_state.cursor_icon = CursorIcon::default();
            app_state.last_press = None;
            app_state.last_release = None;
            app_state.action = Action::StartScreenshot;
	    app_state.active_window = 1;
	    app_state.windows[0].set_visible(true);
	    if render_state.context.is_current() {
		println!("the context for primary window is current");
	    }
	    // render_state.context.make_not_current();
            let window_attrs = Window::default_attributes()
                .with_position(PhysicalPosition::new(x, y))
                .with_inner_size(PhysicalSize::new(w, h));
	    
	    let (window, context, surface, canvas) =
		render::initialize_gl(event_loop, window_attrs);

	    render_state.context = context;
            render_state.surface = surface;
            render_state.canvas = canvas;
	    println!("creating new window with id : {:?}", window.id());
            app_state.windows.push(window);
            app_state.windows[app_state.active_window].request_redraw();
        }
        _ => app_state.windows[app_state.active_window].request_redraw(),
    }
}
