use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowAttributes};

use femtovg::renderer::OpenGl;
use femtovg::Canvas;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::DisplayBuilder;
use glutin_winit::GlWindow;

pub struct RenderState {
    pub context: PossiblyCurrentContext,
    pub surface: Surface<WindowSurface>,
    pub canvas: Canvas<OpenGl>,
}

pub fn initialize_gl(
    event_loop: &ActiveEventLoop,
    window_attrs: WindowAttributes,
) -> (
    Window,
    PossiblyCurrentContext,
    Surface<WindowSurface>,
    Canvas<OpenGl>,
) {
    let window_attrs = window_attrs.with_visible(false);
    let template = ConfigTemplateBuilder::new().with_alpha_size(8);
    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attrs.clone()));
    let (mut window, gl_config) = display_builder
        .build(event_loop, template, |mut config| config.next().unwrap())
        .unwrap();

    let raw_window_handle = window
        .as_ref()
        .and_then(|window| window.window_handle().ok())
        .map(|handle| handle.as_raw());
    let gl_display = gl_config.display();
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);
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

    let window = window.take().unwrap_or_else(|| {
        glutin_winit::finalize_window(event_loop, window_attrs, &gl_config)
            .expect("failed to create the window")
    });

    let attrs = window
        .build_surface_attributes(Default::default())
        .expect("Failed to build surface attributes");

    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };
    let gl_context = not_current_gl_context
        .take()
        .unwrap()
        .make_current(&gl_surface)
        .unwrap();
    let renderer =
        unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s) as *const _) }
            .expect("Cannot create OpenGl renderer");

    let canvas = Canvas::new(renderer).expect("Cannot create the canvas");
    gl_surface
        .swap_buffers(&gl_context)
        .expect("Could not swap buffers");

    window.set_visible(true);
    (window, gl_context, gl_surface, canvas)
}
