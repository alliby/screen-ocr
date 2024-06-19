use crate::action;
use crate::draw;
use crate::render;
use crate::render::RenderState;
use crate::state;
use crate::state::{AppState, WindowState};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, WindowId};

use femtovg::Color;

use glutin::prelude::*;

pub struct App {
    pub render_state: RenderState,
    pub app_state: AppState,
    pub window_state: WindowState,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, context, surface, mut canvas) = render::initialize_gl(event_loop);
        window.set_cursor(self.window_state.cursor_icon);
        let size = window.inner_size();
        self.window_state.cursor_position = (size.width as f32, size.height as f32);
        self.app_state.fonts.push(
            canvas
                .add_font_mem(include_bytes!("../amiri-regular.ttf"))
                .unwrap(),
        );
        self.render_state.window = Some(window);
        self.render_state.context = Some(context);
        self.render_state.surface = Some(surface);
        self.render_state.canvas = Some(canvas);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut action = action::Action::None;
        let (Some(window), Some(context), Some(surface), Some(canvas)) = (
            self.render_state.window.as_mut(),
            self.render_state.context.as_ref(),
            self.render_state.surface.as_ref(),
            self.render_state.canvas.as_mut(),
        ) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                self.window_state.width = size.width as f32;
                self.window_state.height = size.height as f32;
                let dpi_factor = window.scale_factor();
                // Make sure the canvas has the right size
                canvas.set_size(size.width, size.height, dpi_factor as f32);
                canvas.clear_rect(0, 0, size.width, size.height, Color::rgba(0, 0, 0, 0));
                // Draw
                action = draw::app(canvas, &self.app_state, &self.window_state);
                // Tell renderer to execute all drawing commands
                canvas.flush();
                // Notify winit that we're about to submit buffer to the windowing system.
                window.pre_present_notify();
                // Display what we've just rendered
                surface
                    .swap_buffers(context)
                    .expect("Could not swap buffers");
            }
            WindowEvent::Resized(size) => {
                surface.resize(
                    context,
                    size.width.try_into().unwrap(),
                    size.height.try_into().unwrap(),
                );
                action = action::Action::Redraw;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.window_state.cursor_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, .. } => {
                action = state::handle_mouse_input(state, &self.window_state, &mut self.app_state);
            }
            _ => (),
        }

        action::handle_action(action, self);
    }
}
