pub mod app;
pub mod draw;
mod wayland;

use std::time::Instant;

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color};

use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;

use raw_window_handle::HasRawWindowHandle;
use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::keyboard::{Key, NamedKey};
use winit::platform::x11::WindowBuilderExtX11;
use winit::platform::x11::XWindowType;
use winit::window::{Window, WindowBuilder, WindowButtons, WindowLevel};

fn main() {
    run();
}

fn run() {
    let key = "WAYLAND_DISPLAY";
    match std::env::var_os(key) {
        Some(_) => wayland::run(),
        None => winit_run(),
    }
}

fn winit_run() {
    let event_loop = EventLoopBuilder::new().build().unwrap();
    let (context, gl_display, surface, window) = create_window(&event_loop);
    let renderer =
        unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s) as *const _) }
            .expect("Cannot create renderer");

    let canvas = Canvas::new(renderer).expect("Cannot create canvas");

    render(canvas, context, event_loop, surface, window);
}

fn create_window(
    event_loop: &EventLoop<()>,
) -> (
    PossiblyCurrentContext,
    Display,
    Surface<WindowSurface>,
    Window,
) {
    let window_builder = WindowBuilder::new()
        .with_resizable(false)
        .with_decorations(false)
        .with_transparent(true)
        .with_enabled_buttons(WindowButtons::empty())
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_x11_window_type(vec![XWindowType::Dock]);

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, gl_config_picker)
        .unwrap();

    println!("Picked a config with {} samples", gl_config.num_samples());

    let window = window.expect("cannot build window");
    let raw_window_handle = Some(window.raw_window_handle());

    // XXX The display could be obtained from any object created by it, so we can
    // query it from the config.
    let gl_display = gl_config.display();

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    });

    let mut monitor_size = PhysicalSize::new(600, 600);
    for monitor in window.available_monitors() {
        monitor_size = monitor_size.max(monitor.size());
    }
    let _ = window.request_inner_size(monitor_size);
    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        monitor_size.width.try_into().unwrap(),
        monitor_size.height.try_into().unwrap(),
    );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    (
        not_current_gl_context
            .take()
            .unwrap()
            .make_current(&surface)
            .unwrap(),
        gl_display,
        surface,
        window,
    )
}

fn render(
    mut canvas: Canvas<OpenGl>,
    context: PossiblyCurrentContext,
    event_loop: EventLoop<()>,
    surface: Surface<WindowSurface>,
    window: Window,
) {
    let size = window.inner_size();
    let mut app = app::AppState {
        start: Instant::now(),
        width: size.width,
        height: size.height,
        fonts: vec![canvas
            .add_font_mem(include_bytes!("../amiri-regular.ttf"))
            .expect("Cannot add font")],
        start_sel: false,
        sel_corners: (None, None),
    };
    event_loop
        .run(move |event, target| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        app.width = size.width;
                        app.height = size.height;
                        surface.resize(
                            &context,
                            size.width.try_into().unwrap(),
                            size.height.try_into().unwrap(),
                        );
                    }
                    WindowEvent::RedrawRequested => {
                        // Make sure the canvas has the right size:
                        let dpi_factor = window.scale_factor();
                        canvas.set_size(app.width, app.height, dpi_factor as f32);
                        canvas.clear_rect(0, 0, app.width, app.height, Color::rgba(0, 0, 0, 0));

                        draw::app(&mut canvas, &app);
                        // window.request_redraw();

                        // Tell renderer to execute all drawing commands
                        canvas.flush();

                        // Display what we've just rendered
                        surface
                            .swap_buffers(&context)
                            .expect("Could not swap buffers");
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => target.exit(),
                    _ => (),
                },
                _ => (),
            }
        })
        .unwrap()
}

pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}
