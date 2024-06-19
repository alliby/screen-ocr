pub mod action;
pub mod app;
pub mod draw;
pub mod render;
pub mod state;

use app::App;
use render::RenderState;
use state::{AppState, WindowState};

use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // create and run the App
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        app_state: AppState::new(),
        render_state: RenderState::default(),
        window_state: WindowState::new(),
    };
    event_loop.run_app(&mut app).unwrap();
}
