use crate::scenes::RotatedRect;

use vello::kurbo::{Point, Rect};
use vello::peniko::Blob;
use vello::Scene;

use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use winit::window::CursorIcon;

// elements indices
// Area Select elements
pub const FULL_SCREEN_OVERLAY: usize = 0;
pub const SELECTED_RECT: usize = 1;
pub const CONFIRM_BTN: usize = 2;
pub const TOP_LEFT_BTN: usize = 3;
pub const TOP_RIGHT_BTN: usize = 4;
pub const BOTTOM_RIGHT_BTN: usize = 5;
pub const BOTTOM_LEFT_BTN: usize = 6;

// Text Extraction elements
pub static EXTRACTED_ELEMS: Mutex<Option<(Vec<RotatedRect>, String)>> = Mutex::new(None);

#[derive(Default, Debug, Clone, Copy)]
pub struct ViewElement {
    pub bound: Rect,
    pub active: bool,
    pub cursor: CursorIcon,
    pub mouse_enter: bool,
    pub mouse_press: bool,
}

#[derive(Default, Clone)]
pub struct View {
    pub mouse_position: Point,
    pub scene: Scene,
    pub elems: Vec<ViewElement>,
}

#[derive(Default, Clone)]
pub struct AppState {
    pub page: Page,
    pub page_data: Box<PageData>,
    pub redraw: bool,
    pub damaged: bool,
    pub should_exit: bool,
    pub screen_width: f64,
    pub screen_height: f64,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub enum Page {
    #[default]
    AreaSelect,
    TextExtract,
}

#[derive(Debug, Clone, Default)]
pub struct AreaSelectData {
    pub grab: Option<Point>,
    pub resize: Option<usize>,
    pub rect: Rect
}

#[derive(Debug, Clone)]
pub struct TextExtractData {
    pub rect: Rect,
    pub time: Instant,
    pub extracted: bool,
    pub window_cleared: bool,
    pub window_created: bool,
    pub text: String,
    pub rotated_rects: Vec<RotatedRect>,
    pub blob: Blob<u8>,
}

#[derive(Debug, Clone)]
pub enum PageData {
    AreaSelect(AreaSelectData),
    TextExtract(TextExtractData)
}

impl Default for PageData {
    fn default() -> Self {
        Self::AreaSelect(AreaSelectData::default())
    }
}

impl AppState {
    pub fn callbacks(&self) -> Vec<fn(&mut AppState, &mut View, usize)> {
        match self.page {
            Page::AreaSelect => {
                let mut callbacks: Vec<fn(&mut AppState, &mut View, usize)> = vec![];

                // the full screen overlay
                callbacks.push(|state, view, _| {
                    let mouse = view.mouse_position;
                    let PageData::AreaSelect(ref mut page_data) = *state.page_data else {
                        return;
                    };
                    // if no press the second call is for the mouse released
                    let mouse_press = view.elems[FULL_SCREEN_OVERLAY].mouse_press;
                    if mouse_press {
                        view.elems[SELECTED_RECT].bound.x0 = mouse.x;
                        view.elems[SELECTED_RECT].bound.y0 = mouse.y;
                        view.elems[CONFIRM_BTN].bound = Rect::ZERO;
                        for i in SELECTED_RECT..=BOTTOM_LEFT_BTN {
                            view.elems[i].active = false;
                        }
                    } else {
                        view.elems[CONFIRM_BTN].active = true;
                        view.elems[SELECTED_RECT].active = true;
                        page_data.rect = view.elems[SELECTED_RECT].bound;
                        for i in SELECTED_RECT..=BOTTOM_LEFT_BTN {
                            view.elems[i].active = true;
                        }
                    }
                });

                // for the selected rect
                callbacks.push(|state, view, _| {
                    let mouse = view.mouse_position;
                    let PageData::AreaSelect(ref mut page_data) = *state.page_data else {
                        return;
                    };

                    if page_data.grab.is_none() {
                        page_data.grab = Some(mouse);
                    } else {
                        page_data.grab = None;
                        page_data.rect = view.elems[SELECTED_RECT].bound;
                    }
                });

                // for the confirm button
                callbacks.push(|state, _, _| {
                    let PageData::AreaSelect(ref page_data) = *state.page_data else {
                        return;
                    };
                    state.damaged = true;
                    state.redraw = true;
                    state.page = Page::TextExtract;
                    state.page_data = Box::new(PageData::TextExtract(TextExtractData {
                        rect: page_data.rect,
                        time: Instant::now(),
                        window_cleared: false,
                        window_created: false,
			rotated_rects: Vec::new(),
			text: String::new(),
                        extracted: false,
                        blob: Blob::new(Arc::new([])),
                    }));
                });

                // Resize Buttons Callbacks
                for _ in TOP_LEFT_BTN..=BOTTOM_LEFT_BTN {
                    callbacks.push(|state, view, index| {
			let PageData::AreaSelect(ref mut page_data) = *state.page_data else {
                            return;
			};

                        if page_data.resize.is_none() {
                            page_data.resize = Some(index);
                        } else {
                            page_data.resize = None;
                            page_data.rect = view.elems[SELECTED_RECT].bound;
                        }
                    });
                }
                // return the callbacks vec
                callbacks
            }
            Page::TextExtract => vec![
		|_, _, index| {
		    println!("input on {index}");
		}
	    ],
        }
    }

    pub fn view_elements(&self) -> Vec<ViewElement> {
        match self.page {
            Page::AreaSelect => {
                let mut views = vec![];
                // full screen overlay
                views.push(ViewElement {
                    cursor: CursorIcon::Crosshair,
                    bound: Rect::new(0.0, 0.0, self.screen_width, self.screen_height),
                    active: true,
                    ..Default::default()
                });
                // Selected Rectangle
                views.push(ViewElement {
                    cursor: CursorIcon::Grab,
                    ..Default::default()
                });
                // push the remaining elements (confirm button + resize buttons)
                for _ in CONFIRM_BTN..=BOTTOM_RIGHT_BTN + 1 {
                    views.push(ViewElement {
                        cursor: CursorIcon::Pointer,
                        ..Default::default()
                    });
                }
                // return the views vec
                views
            }
            Page::TextExtract => vec![],
        }
    }
}

// Extract Text from image bytes and write it to the global Cell
pub fn extract_text(blob: Blob<u8>, dimensions: (u32, u32)) {
    let detection_model = Model::load_file("assets/text-detection.rten").unwrap();
    let recognition_model = Model::load_file("assets/text-recognition.rten").unwrap();
    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap();
    let img_source = ImageSource::from_bytes(blob.data(), dimensions).unwrap();
    let ocr_input = engine.prepare_input(img_source).unwrap();
    let word_rects = engine.detect_words(&ocr_input).unwrap();
    // sort the words in lines and then flatten the lines into words again
    let rects = engine
        .find_text_lines(&ocr_input, &word_rects)
        .into_iter()
        .flatten()
        .map(RotatedRect::from);
    let text = engine.get_text(&ocr_input).unwrap();
    let mut extracted = EXTRACTED_ELEMS.lock().unwrap();
    *extracted = Some((rects.collect(), text));
}
