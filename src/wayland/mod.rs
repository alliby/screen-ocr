pub mod output;
pub mod state;
pub mod screenshot;

use crate::app::AppState;
use state::WaylandState;

use femtovg::renderer::OpenGl;
use femtovg::Canvas;

use std::convert::TryInto;
use std::time::Instant;

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::{pointer::CursorIcon, SeatState},
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell},
        WaylandSurface,
    },
    shm::Shm,
};

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::{Display, DisplayApiPreference, GetGlDisplay};
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};

use wayland_client::{globals::registry_queue_init, Connection, Proxy};

pub fn run() {
    // Connect to the Wayland server
    let conn = Connection::connect_to_env().unwrap();
    let mut wayland_display_handle = WaylandDisplayHandle::empty();
    wayland_display_handle.display = conn.backend().display_ptr() as *mut _;
    let (width, height) = output::monitor_size(&conn).unwrap();
    println!("{width} x {height}");

    // Set up the registry and event queue for Wayland
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    // Get the required Wayland globals (compositor and layer shell)
    let compositor_state =
        CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm_state = Shm::bind(&globals, &qh).expect("wl_shm not available");

    // Create a Wayland surface and layer surface
    let surface = compositor_state.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface.clone(),
        Layer::Overlay,
        Some("trans_screen"),
        None,
    );
    layer.set_anchor(Anchor::all());
    layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer.set_size(width, height);
    layer.commit();

    // Set up the window handle and display handle for glutin
    let mut wayland_window_handle = WaylandWindowHandle::empty();
    wayland_window_handle.surface = surface.id().as_ptr() as *mut _;
    let raw_display_handle = RawDisplayHandle::Wayland(wayland_display_handle);
    let raw_window_handle = RawWindowHandle::Wayland(wayland_window_handle);

    // Configure the glutin template builder for OpenGL
    let template_builder = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .compatible_with_native_window(raw_window_handle);

    // Create the glutin display and find a suitable OpenGL config
    let gl_display =
        unsafe { Display::new(raw_display_handle, DisplayApiPreference::Egl).unwrap() };
    let template = template_builder.build();
    let gl_config = unsafe {
        let configs = gl_display.find_configs(template).unwrap();
        crate::gl_config_picker(configs)
    };

    println!("Picked a config with {} samples", gl_config.num_samples());

    // Set up the OpenGL context attributes and create the context
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(Some(raw_window_handle));

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

    // Create the glutin window surface
    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        width.try_into().unwrap(),
        height.try_into().unwrap(),
    );
    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    // Make the OpenGL context current and create the renderer
    let context = not_current_gl_context
        .take()
        .unwrap()
        .make_current(&surface)
        .unwrap();
    let renderer =
        unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s) as *const _) }
            .expect("Cannot create renderer");

    // Create the canvas
    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    let app = AppState {
        start: Instant::now(),
        width,
        height,
        fonts: vec![canvas
            .add_font_mem(include_bytes!("../../amiri-regular.ttf"))
            .expect("Cannot add font")],
        start_sel: false,
        sel_corners: (None, None),
    };

    let mut wayland_state = WaylandState {
        app,
        compositor_state,
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm_state,

        exit: false,
        first_configure: true,
        layer,
        canvas,
        surface,
        context,
        keyboard: None,
        themed_pointer: None,
        cursor_icon: CursorIcon::Crosshair,
        set_cursor: false,
    };

    // Event loop
    loop {
        event_queue.blocking_dispatch(&mut wayland_state).unwrap();

        if wayland_state.exit {
            println!("exiting example");
            break;
        }
    }

    // On exit we must destroy the surface before the window is destroyed.
    drop(wayland_state.surface);
    drop(wayland_state.layer);
}
