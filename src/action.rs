use crate::app::App;
use winit::window::CursorIcon;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    // ButtonHover,
    Redraw,
    None,
}

pub fn handle_action(action: Action, app: &mut App) {
    let Some(window) = app.render_state.window.as_ref() else{ return };
    match action {
        Action::Redraw => window.request_redraw(),
        // ButtonHover => window.set_cursor(CursorIcon::Pointer),
        Action::None => {}
    }
}
