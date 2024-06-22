use crate::draw;
use crate::render;
use crate::render::RenderState;
use crate::state;
use crate::state::AppState;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use femtovg::Color;

use glutin::prelude::*;

#[derive(Default)]
pub struct App {
    pub render: Option<RenderState>,
    pub state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, context, surface, mut canvas) = render::initialize_gl(event_loop);
        let mut app_state = AppState::new(window);
        // window.set_cursor(app_state.cursor_icon);
        app_state.fonts.push(
            canvas
                .add_font_mem(include_bytes!("../amiri-regular.ttf"))
                .unwrap(),
        );

        self.state = Some(app_state);
        self.render = Some(RenderState {
            context,
            surface,
            canvas,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(render), Some(mut app_state)) = (self.render.as_mut(), self.state.as_mut())
        else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let size = app_state.window.inner_size();
                let dpi_factor = app_state.window.scale_factor();
                let (width, height) = (size.width as u32, size.height as u32);
                // Make sure the canvas has the right size
                render.canvas.set_size(width, height, dpi_factor as f32);
                render
                    .canvas
                    .clear_rect(0, 0, width, height, Color::rgba(0, 0, 0, 0));
                // handle app state
                state::handle_state(&mut app_state);
                // Draw
                draw::app(&mut render.canvas, &mut app_state);
                // Tell renderer to execute all drawing commands
                render.canvas.flush();
                // Notify winit that we're about to submit buffer to the windowing system.
                app_state.window.pre_present_notify();
                // Display what we've just rendered
                render
                    .surface
                    .swap_buffers(&render.context)
                    .expect("Could not swap buffers");
            }
            WindowEvent::Resized(size) => {
                render.surface.resize(
                    &render.context,
                    size.width.try_into().unwrap(),
                    size.height.try_into().unwrap(),
                );
                app_state.window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                app_state.cursor_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, .. } => match state {
                ElementState::Pressed => {
                    app_state.last_press = Some(app_state.cursor_position);
                    app_state.last_release = None;
                }
                ElementState::Released => {
                    app_state.last_release = Some(app_state.cursor_position);
                    app_state.last_press = None;
                }
            },
            _ => (),
        }
    }
}
