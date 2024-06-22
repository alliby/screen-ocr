pub mod app;
pub mod draw;
pub mod render;
pub mod state;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // create and run the App
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
