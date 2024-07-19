use crate::app::App;
use crate::capture;
use crate::draw::ImageKey;
use crate::helper::Rectangle;
use crate::render::{self, RenderState};

use femtovg::imgref::ImgVec;
use femtovg::rgb::RGB8;
use femtovg::{ImageFilter, ImageFlags, PixelFormat};

use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use rten_imageproc::RotatedRect;

use std::sync::OnceLock;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, Window, WindowLevel};

#[cfg(target_os = "linux")]
use winit::platform::x11::WindowAttributesExtX11;

type Point = (f32, f32);

// this cell is used for the text extraction which happen on other thread
static CELL: OnceLock<Vec<(Vec<RotatedRect>, String)>> = OnceLock::new();

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
                app.windows[app.active].request_redraw();
            }
            if let Some(cursor_position) = app.last_release {
                let Some(point_1) = state.start_point else {
                    return;
                };
                app.action = Action::AreaConfirm(AreaConfirmState {
                    confirm: false,
                    corners: (point_1, cursor_position),
                });
                app.windows[app.active].request_redraw();
            }
        }

        Action::AreaConfirm(AreaConfirmState {
            confirm: true,
            corners,
        }) => {
            // get the rectangle of the selected area
            let img_rect = Rectangle::from(corners);
            let window_rect = Rectangle::new(
                (img_rect.x - 10.0).max(0.0),
                (img_rect.y - 10.0).max(0.0),
                (img_rect.width + 20.0).clamp(120.0, app.width),
                (img_rect.height + 20.0).clamp(120.0, app.height),
            );

            // close the selection overlay
            app.windows[0].set_visible(false);
            app.active = 1;

            // Create the new window
            let window_position = PhysicalPosition::new(
                (app.width - window_rect.width) / 2.0,
                (app.height - window_rect.height) / 2.0,
            );
            let window_size = PhysicalSize::new(window_rect.width, window_rect.height);
            #[allow(unused_mut)]
            let mut window_attrs = Window::default_attributes()
                .with_decorations(false)
                .with_window_level(WindowLevel::AlwaysOnTop)
                .with_position(window_position)
                .with_inner_size(window_size);

            #[cfg(target_os = "linux")]
            {
                window_attrs = window_attrs.with_override_redirect(true);
            }

            let (window, context, surface, mut canvas) =
                render::initialize_gl(event_loop, window_attrs);

            (app.last_press, app.last_release) = (None, None);
            (app.width, app.height) = (window_rect.width, window_rect.height);
            app.cursor_icon = CursorIcon::default();

            // Capture the screen and write the image buffer to the app
            app.screenshot = capture::screen_rect(img_rect).ok();

            // Add the screenshot image to the canvas
            let img_flags = ImageFlags::NEAREST;
            let image_state = app
                .screenshot
                .as_ref()
                .map(|bytes| {
                    let img = bytes.chunks(3).map(|p| RGB8::new(p[0], p[1], p[2]));
                    let src = ImgVec::new(
                        img.collect(),
                        img_rect.width as usize,
                        img_rect.height as usize,
                    );
                    let img_id = canvas.create_image(src.as_ref(), img_flags).unwrap();

                    // Create a new blurry image
                    let filter = ImageFilter::GaussianBlur { sigma: 1.0 };
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
            let canvas = &mut app.render[app.active].canvas;
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
                let word_rects = engine.detect_words(&ocr_input).unwrap();
                let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
                let line_text = engine
                    .recognize_text(&ocr_input, &line_rects)
                    .unwrap()
                    .into_iter()
                    .map(|line| line.map(|l| l.to_string()).unwrap_or_default());
                let collection = line_rects.into_iter().zip(line_text);
                CELL.set(collection.collect()).unwrap();
            });
        }

        Action::ExtractText(ref mut state) => match CELL.get() {
            Some(collection) => {
                static mut REDRAW: bool = true;
                app.line_rects = collection.to_vec();
                *state = ExtractionState::Extracted;
                if unsafe { REDRAW } {
                    app.windows[app.active].request_redraw();
                    unsafe {
                        REDRAW = false;
                    }
                }
            }
            None => {
                *state = ExtractionState::Extracting;
                app.windows[app.active].request_redraw();
            }
        },
        _ => {
            app.windows[app.active].request_redraw();
        }
    }
}
