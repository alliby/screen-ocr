use crate::draw;
use crate::render;
use crate::render::RenderState;
use crate::state;
use crate::state::AppState;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

use femtovg::Color;

use glutin::prelude::*;

#[derive(Default)]
pub struct App {
    pub render: Option<RenderState>,
    pub state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        let screen_size = event_loop
            .primary_monitor()
            .map(|monitor| monitor.size())
            .expect("Cannot get the primary monitor");
        let mut window_attrs = Window::default_attributes()
            .with_cursor(CursorIcon::Crosshair)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(0.0, 0.0))
            .with_inner_size(screen_size)
            .with_transparent(true)
            .with_resizable(false)
            .with_decorations(false);

        #[cfg(target_os = "linux")]
        {
            window_attrs = window_attrs.with_override_redirect(true);
        }

        #[cfg(target_os = "windows")]
        {
            // in windows when the inner size is the same as the screen size
            // with the top left position they create standard full screen size with no transparency
            let screen_size = PhysicalSize::new(
                screen_size.width as f32 + 1.0,
                screen_size.height as f32 + 1.0,
            );
            window_attrs = window_attrs
                .with_inner_size(screen_size)
                .with_skip_taskbar(true);
        }

        let (window, context, surface, mut canvas) =
            render::initialize_gl(event_loop, window_attrs);
        let mut app_state = AppState::new(window);
        app_state.width = screen_size.width as f32;
        app_state.height = screen_size.height as f32;
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // unwrap render and app state from the main app
        let (Some(mut render), Some(mut app_state)) = (self.render.as_mut(), self.state.as_mut())
        else {
            return;
        };
        if id != app_state.windows[app_state.active_window].id() {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let dpi_factor = app_state.windows[app_state.active_window].scale_factor() as f32;
                let (width, height) = (app_state.width as u32, app_state.height as u32);
                // Make sure the canvas has the right size
                render.canvas.set_size(width, height, dpi_factor);
                render
                    .canvas
                    .clear_rect(0, 0, width, height, Color::rgba(0, 0, 0, 0));
                // Draw
                draw::app(&mut render.canvas, &mut app_state);
                // Tell renderer to execute all drawing commands
                render.canvas.flush();
                // Notify winit that we're about to submit buffer to the windowing system.
                app_state.windows[app_state.active_window].pre_present_notify();
                // Display what we've just rendered
                render
                    .surface
                    .swap_buffers(&render.context)
                    .expect("Could not swap buffers");
            }
            WindowEvent::Resized(size) => {
		app_state.width = size.width as f32;
		app_state.height = size.height as f32;
                render.surface.resize(
                    &render.context,
                    size.width.try_into().unwrap(),
                    size.height.try_into().unwrap(),
                );
                app_state.windows[app_state.active_window].request_redraw();
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

        // after handling the window event we handle the app and render state
        state::handle_state(&mut app_state, &mut render, event_loop);
   }
}
