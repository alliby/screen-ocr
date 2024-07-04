use crate::app::App;
use crate::capture;
use crate::draw::ImageKey;
use crate::helper::Rectangle;
use crate::render::{self, RenderState};

use anyhow::Result;

use femtovg::imgref::ImgVec;
use femtovg::rgb::RGB8;
use femtovg::ImageFlags;

use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

use std::sync::OnceLock;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, Window};

#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;

#[cfg(target_os = "linux")]
use winit::platform::x11::WindowAttributesExtX11;

type Point = (f32, f32);

// this cell is used for the text extraction which happen on other thread
static CELL: OnceLock<Result<String>> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub struct AreaSelectState {
    pub start_point: Option<Point>,
}

#[derive(Debug, Clone, Copy)]
pub struct AreaConfirmState {
    pub confirm: bool,
    pub corners: (Point, Point),
}

#[derive(Debug, Clone, Copy)]
pub enum ImageState {
    Loaded,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub enum ExtractionState {
    Extracted,
    Extracting,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    AreaSelect(AreaSelectState),
    AreaConfirm(AreaConfirmState),
    GetScreenshot(ImageState),
    ExtractText(ExtractionState),
}

pub fn handle_state(app: &mut App, event_loop: &ActiveEventLoop) {
    if app.should_exit {
        event_loop.exit();
        return;
    }

    match app.action {
        Action::AreaSelect(ref mut state) => {
            if app.last_press.is_some() {
                state.start_point = app.last_press;
            }
            if let Some(cursor_position) = app.last_release {
                let Some(point_1) = state.start_point else {
                    return;
                };
                app.action = Action::AreaConfirm(AreaConfirmState {
                    confirm: false,
                    corners: (point_1, cursor_position),
                });
            }
            app.windows[app.active].request_redraw();
        }

        Action::AreaConfirm(AreaConfirmState {
            confirm: true,
            corners,
        }) => {
            // get the rectangle of the selected area
            let rect = Rectangle::from(corners);
            // close the selection overlay
            app.windows[0].set_visible(false);
            app.active = 1;
            // Create the new window
            (app.last_press, app.last_release) = (None, None);
            (app.width, app.height) = (rect.width, rect.height);
            app.cursor_icon = CursorIcon::default();
            let mut window_attrs = Window::default_attributes()
                .with_decorations(false)
                .with_position(PhysicalPosition::new(rect.x, rect.y))
                .with_inner_size(PhysicalSize::new(rect.width, rect.height));

            #[cfg(target_os = "windows")]
            {
                window_attrs = window_attrs.with_skip_taskbar(true);
            }

            #[cfg(target_os = "linux")]
            {
                window_attrs = window_attrs.with_override_redirect(true);
            }

            let (window, context, surface, mut canvas) =
                render::initialize_gl(event_loop, window_attrs);
            // Capture the screen and write the image buffer to the app
            app.screenshot = capture::capture_screen(rect).ok();
            // Add the screenshot image to the canvas
            let img_flags = ImageFlags::empty();
            let image_state = app
                .screenshot
                .as_ref()
                .map(|bytes| {
                    let img = bytes.chunks(3).map(|p| RGB8::new(p[0], p[1], p[2]));
                    let src = ImgVec::new(img.collect(), rect.width as usize, rect.height as usize);
                    let img_id = canvas.create_image(src.as_ref(), img_flags).unwrap();

                    // Create a new blurry image
                    let filter = ImageFilter::GaussianBlur { sigma: 20.0 };
                    let blur_img_id = canvas
                        .create_image_empty(src.width(), src.height(), PixelFormat::Rgb8, img_flags)
                        .unwrap();
                    canvas.filter_image(blur_img_id, filter, img_id);

                    app.img_ids.insert(ImageKey::Screenshot, img_id);
                    app.img_ids.insert(ImageKey::BluredScreen, blur_img_id);
                    ImageState::Loaded
                })
                .unwrap_or(ImageState::Error);
            app.action = Action::GetScreenshot(image_state);
            // Add the new RenderState to the app
            app.render.push(RenderState {
                context,
                surface,
                canvas,
            });
            app.windows.push(window);
            app.windows[app.active].request_redraw();
        }

        Action::GetScreenshot(ImageState::Loaded) => {
            let canvas = &app.render[app.active].canvas;
            let img_id = app.img_ids[&ImageKey::Screenshot];
            let img_dim = canvas.image_size(img_id).unwrap();
            let img_dim = (img_dim.0 as u32, img_dim.1 as u32);
            let img_buff = app.screenshot.as_ref().cloned().unwrap();
            app.action = Action::ExtractText(ExtractionState::Extracting);
            std::thread::spawn(move || {
                let detection_model = Model::load_file("text-detection.rten").unwrap();
                let recognition_model = Model::load_file("text-recognition.rten").unwrap();
                let engine = OcrEngine::new(OcrEngineParams {
                    detection_model: Some(detection_model),
                    recognition_model: Some(recognition_model),
                    ..Default::default()
                })
                .unwrap();
                let img_source = ImageSource::from_bytes(&img_buff[..], img_dim).unwrap();
                let ocr_input = engine.prepare_input(img_source).unwrap();
                CELL.set(engine.get_text(&ocr_input)).unwrap();
            });
        }

        Action::ExtractText(ref mut state) => match CELL.get() {
            Some(Ok(extracted_text)) => {
                app.extracted_text = Some(extracted_text.clone());
                *state = ExtractionState::Extracted;
                app.windows[app.active].request_redraw();
            }
            Some(Err(_)) => *state = ExtractionState::Error,
            None => {
                *state = ExtractionState::Extracting;
                app.windows[app.active].request_redraw();
            }
        },
        _ => app.windows[app.active].request_redraw(),
    }
}
