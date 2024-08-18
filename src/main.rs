pub mod app;
pub mod scenes;
pub mod state;
pub mod capture;

use anyhow::Result;
use vello::util::RenderContext;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    let mut app = app::App {
        context: RenderContext::new(),
        active: 0,
        renderers: vec![],
        surfaces: vec![],
        windows: vec![],
        state: Default::default(),
        view: Default::default(),
        callbacks: vec![],
    };

    // Create and run a winit event loop
    let event_loop = EventLoop::new()?;
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
    Ok(())
}
