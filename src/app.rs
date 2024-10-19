use crate::capture;
use crate::scenes;
use crate::state::*;

use std::num::NonZeroUsize;
use std::sync::Arc;
use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions, Scene};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::*;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowLevel};

#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;

#[cfg(target_os = "linux")]
use winit::platform::x11::WindowAttributesExtX11;

use vello::wgpu;

const OVERLAY_WINDOW: usize = 0;
const EXTRACT_WINDOW: usize = 1;

pub struct App<'s> {
    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    pub context: RenderContext,
    // An array of renderers, one per wgpu device
    pub active: usize,
    pub renderers: Vec<Option<Renderer>>,
    pub surfaces: Vec<RenderSurface<'s>>,
    pub windows: Vec<Arc<Window>>,
    // the App states
    pub view: View,
    pub state: AppState,
    pub callbacks: Vec<fn(&mut AppState, &mut View, usize)>,
}

impl<'s> App<'s> {
    #[inline]
    fn callback(&self, index: usize) -> fn(&mut AppState, &mut View, usize) {
        match self.state.page {
            Page::AreaSelect => self.callbacks[index],
            Page::TextExtract => self.callbacks[0],
        }
    }
}

impl<'s> ApplicationHandler for App<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Get the winit window cached in a previous Suspended event or else create a new window
        let window = Arc::new(create_overlay_window(event_loop));

        // Create a vello Renderer for the surface (using its device id)
        let surface = create_vello_surface(window.clone(), &mut self.context);
        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        // show the window when the initialization finish
        let size = window.inner_size();
        self.state.screen_width = size.width as f64;
        self.state.screen_height = size.height as f64;
        window.set_visible(true);

        // Push the Window and Surface to App
        self.windows.push(window.clone());
        self.surfaces.push(surface);

        // set the control flow for Power-saving reactive rendering
        event_loop.set_control_flow(ControlFlow::Wait);

        // damage the state for the first rendering
        self.state.damaged = true;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.state.should_exit {
            event_loop.exit()
        }

        if self.state.damaged {
            self.view.scene = Scene::new();
            self.callbacks = self.state.callbacks();
            self.view.elems = self.state.view_elements();
            self.state.damaged = false;
        }

        if self.windows[self.active].id() != window_id {
            return;
        }

        if let PageData::TextExtract(ref mut page_data) = *self.state.page_data {
            if page_data.window_cleared && !page_data.window_created {
                // Capture the screen after clearing the overlay
                page_data.blob = capture::screen_rect(page_data.rect).unwrap();
                let dim = (
                    page_data.rect.width().abs() as u32,
                    page_data.rect.height().abs() as u32,
                );
                let img_blob = page_data.blob.clone();
                std::thread::spawn(move || extract_text(img_blob, dim));
                // Create a new window
                self.windows[OVERLAY_WINDOW].set_visible(false);
                let window = Arc::new(create_main_window(event_loop));
                let surface = create_vello_surface(window.clone(), &mut self.context);
                self.surfaces.push(surface);
                self.windows.push(window.clone());
                self.active = EXTRACT_WINDOW;
                page_data.window_created = true;
            }
        }

        let surface = &mut self.surfaces[self.active];

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            // Resize the surface when the window is resized
            WindowEvent::Resized(size) => {
                if size != Default::default() {
                    self.state.screen_width = size.width as f64;
                    self.state.screen_height = size.height as f64;
                    self.context
                        .resize_surface(surface, size.width, size.height);
                    self.windows[self.active].request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                // use the same underlying Scene allocation if the state is not damaged.
                self.view.scene.reset();

                // Re-add the objects to draw to the scene.
                scenes::draw(&mut self.state, &mut self.view);

                // Get the window size
                let width = surface.config.width;
                let height = surface.config.height;

                // Get a handle to the device
                let device_handle = &self.context.devices[surface.dev_id];

                // Get the surface's texture
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                // Render to the surface's texture
                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_surface(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.view.scene,
                        &surface_texture,
                        &vello::RenderParams {
                            base_color: Color::TRANSPARENT, // Background color
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa8,
                        },
                    )
                    .expect("failed to render to surface");

                // Queue the texture to be presented on the surface
                surface_texture.present();

                device_handle.device.poll(wgpu::Maintain::Poll);
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.view.mouse_position = (position.x, position.y).into();
                let mouse = self.view.mouse_position;
                for elm in self.view.elems.iter_mut().filter(|e| e.active).rev() {
                    let entered = elm.mouse_enter;
                    let hover = elm.bound.abs().contains(mouse);
                    elm.mouse_enter = hover;
                    if (hover && !entered) || (!hover && entered) {
                        self.windows[self.active].request_redraw();
                    }
                    if hover {
                        self.windows[self.active].set_cursor(elm.cursor);
                        break;
                    }
                }
            }

            WindowEvent::MouseInput { state, .. } => {
                for i in (0..self.view.elems.len()).rev() {
                    let elem = &mut self.view.elems[i];
                    if elem.mouse_enter && elem.active {
                        elem.mouse_press = state.is_pressed();
                        (self.callback(i))(&mut self.state, &mut self.view, i);
                        self.windows[self.active].request_redraw();
                        break;
                    }
                }
            }
            _ => {}
        }

        // redraw if draw or state callbacks request that
        if self.state.redraw {
            self.windows[self.active].request_redraw();
            self.state.redraw = false;
        }
    }
}

fn create_main_window(event_loop: &ActiveEventLoop) -> Window {
    let attr = Window::default_attributes()
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_min_inner_size(PhysicalSize::new(400.0, 100.0))
        .with_title("Screen OCR");

    event_loop.create_window(attr).unwrap()
}

fn create_overlay_window(event_loop: &ActiveEventLoop) -> Window {
    // TODO: better way for muliple monitors
    let screen_size = event_loop
        .primary_monitor()
        .map(|monitor| monitor.size())
        .expect("Cannot get the primary monitor");

    let mut attr = Window::default_attributes()
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_position(PhysicalPosition::new(0.0, 0.0))
        .with_inner_size(screen_size)
        .with_decorations(false)
        .with_resizable(true)
        .with_transparent(true)
        .with_visible(false)
        .with_title("screen OCR");

    #[cfg(target_os = "linux")]
    {
        attr = attr.with_override_redirect(true);
    }

    #[cfg(target_os = "windows")]
    {
        // this is a small hack for windows platform
        // when the inner size is the same as the screen size
        // with the top left position a black full screen window is created
        let screen_size = PhysicalSize::new(
            screen_size.width as f32 + 1.0,
            screen_size.height as f32 + 1.0,
        );
        attr = attr.with_inner_size(screen_size).with_skip_taskbar(true);
    }

    event_loop.create_window(attr).unwrap()
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("Couldn't create renderer")
}

fn create_vello_surface<'s, 'c>(
    window: Arc<Window>,
    context: &'c mut RenderContext,
) -> RenderSurface<'s> {
    // Create a vello Surface
    let size = window.inner_size();
    let surface_future = context.create_surface(
        window,
        size.width,
        size.height,
        wgpu::PresentMode::AutoVsync,
    );
    pollster::block_on(surface_future).expect("Error creating surface")
}
