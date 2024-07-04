use crate::draw;
use crate::draw::ImageKey;
use crate::render;
use crate::render::RenderState;
use crate::state;
use crate::state::*;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

use femtovg::{Color, FontId, ImageId};
use glutin::prelude::*;

use std::collections::HashMap;
use std::time::Instant;

type Point = (f32, f32);

pub struct App {
    pub active: usize,
    pub render: Vec<RenderState>,
    pub windows: Vec<Window>,
    pub width: f32,
    pub height: f32,
    pub start_time: Instant,
    pub fonts: Vec<FontId>,
    pub img_ids: HashMap<ImageKey, ImageId>,
    pub cursor_position: Point,
    pub last_press: Option<Point>,
    pub last_release: Option<Point>,
    pub screenshot: Option<Vec<u8>>,
    pub extracted_text: Option<String>,
    pub action: Action,
    pub cursor_icon: CursorIcon,
    pub should_exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            render: Vec::new(),
            windows: Vec::new(),
            active: 0,
            width: 800.0,
            height: 600.0,
            start_time: Instant::now(),
            fonts: Vec::new(),
            img_ids: HashMap::new(),
            cursor_position: Default::default(),
            last_press: None,
            last_release: None,
            screenshot: None,
            extracted_text: None,
            action: Action::AreaSelect(AreaSelectState { start_point: None }),
            cursor_icon: CursorIcon::Crosshair,
            should_exit: false,
        }
    }
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
            // with the top left position they create a black full screen window
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

        self.width = screen_size.width as f32;
        self.height = screen_size.height as f32;
        self.fonts.push(
            canvas
                .add_font_mem(include_bytes!("../amiri-regular.ttf"))
                .unwrap(),
        );
        self.render.push(RenderState {
            context,
            surface,
            canvas,
        });
        self.windows.push(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if id != self.windows[self.active].id() {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                // Make sure the canvas has the right size
                let dpi_factor = self.windows[self.active].scale_factor() as f32;
                let (width, height) = (self.width as u32, self.height as u32);
                self.render[self.active]
                    .canvas
                    .set_size(width, height, dpi_factor);
                self.render[self.active].canvas.clear_rect(
                    0,
                    0,
                    width,
                    height,
                    Color::rgba(0, 0, 0, 0),
                );
                // Draw
                draw::app(self);
                // Tell renderer to execute all drawing commands
                self.render[self.active].canvas.flush();
                // Notify winit that we're about to submit buffer to the windowing system.
                self.windows[self.active].pre_present_notify();
                // Display what we've just rendered
                self.render[self.active]
                    .surface
                    .swap_buffers(&self.render[self.active].context)
                    .expect("Could not swap buffers");
            }
            WindowEvent::Resized(size) => {
                self.width = size.width as f32;
                self.height = size.height as f32;
                self.render[self.active].surface.resize(
                    &self.render[self.active].context,
                    size.width.try_into().unwrap(),
                    size.height.try_into().unwrap(),
                );
                self.windows[self.active].request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, .. } => match state {
                ElementState::Pressed => {
                    self.last_press = Some(self.cursor_position);
                    self.last_release = None;
                }
                ElementState::Released => {
                    self.last_release = Some(self.cursor_position);
                    self.last_press = None;
                }
            },
            _ => (),
        }

        // after handling the window event we handle the app and render state
        state::handle_state(self, event_loop);
    }
}
