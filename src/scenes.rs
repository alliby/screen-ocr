use crate::state::*;
use std::f64::consts::PI;
use std::sync::Arc;
use std::time::{Instant, Duration};

use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;

use vello::kurbo::{Affine, CircleSegment, Rect, Stroke};
use vello::peniko::{Blob, Color, Fill, Font};
use vello::skrifa::instance::LocationRef;
use vello::skrifa::raw::FontRef;
use vello::skrifa::MetadataProvider;
use vello::{Glyph, Scene};


const ROBOTO_FONT: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

pub fn draw(state: &mut AppState, view: &mut View) {
    let scene = &mut view.scene;
    match state.page {
        Page::AreaSelect => {
            let mouse = view.mouse_position;
            let PageData::AreaSelect(ref mut page_data) = state.page_data else {
                return;
            };

            if view.elems[FULL_SCREEN_OVERLAY].mouse_press {
                state.redraw = true;
                view.elems[SELECTED_RECT].bound.x1 = mouse.x;
                view.elems[SELECTED_RECT].bound.y1 = mouse.y;
            }

            // move the selected rectangle when dragged
            if let Some(point) = page_data.grab {
                state.redraw = true;
                let translate = page_data.rect + (mouse - point);
                view.elems[SELECTED_RECT].bound = translate;
            }

            // resize the selected rectangle
            if let Some(index) = page_data.resize {
                state.redraw = true;
                if index == TOP_LEFT_BTN {
                    view.elems[SELECTED_RECT].bound.x0 = mouse.x;
                    view.elems[SELECTED_RECT].bound.y0 = mouse.y;
                }
                if index == TOP_RIGHT_BTN {
                    view.elems[SELECTED_RECT].bound.x1 = mouse.x;
                    view.elems[SELECTED_RECT].bound.y0 = mouse.y;
                }
                if index == BOTTOM_RIGHT_BTN {
                    view.elems[SELECTED_RECT].bound.x1 = mouse.x;
                    view.elems[SELECTED_RECT].bound.y1 = mouse.y;
                }
                if index == BOTTOM_LEFT_BTN {
                    view.elems[SELECTED_RECT].bound.x0 = mouse.x;
                    view.elems[SELECTED_RECT].bound.y1 = mouse.y;
                }
            }

            // define the invisible resize buttons bounds
            if view.elems[FULL_SCREEN_OVERLAY].mouse_press
                || view.elems[SELECTED_RECT..].iter().any(|v| v.mouse_press)
            {
                let Rect { x0, y0, x1, y1 } = view.elems[SELECTED_RECT].bound;
                let size = (30.0, 30.0);
                let combinations = [
                    (TOP_LEFT_BTN, Rect::from_center_size((x0, y0), size)),
                    (TOP_RIGHT_BTN, Rect::from_center_size((x1, y0), size)),
                    (BOTTOM_RIGHT_BTN, Rect::from_center_size((x1, y1), size)),
                    (BOTTOM_LEFT_BTN, Rect::from_center_size((x0, y1), size)),
                ];
                for (i, r) in combinations {
                    view.elems[i].bound = r;
                }
            }

            // make sure that rectangle fit the screen
            clamp_width(
                &mut view.elems[SELECTED_RECT].bound,
                0.0,
                state.screen_width,
            );
            clamp_height(
                &mut view.elems[SELECTED_RECT].bound,
                0.0,
                state.screen_height,
            );

            // define the confirm button bound
            if view.elems[FULL_SCREEN_OVERLAY].mouse_press
                || view.elems[SELECTED_RECT..].iter().any(|v| v.mouse_press)
                || page_data.resize.is_some()
            {
                let x = view.elems[SELECTED_RECT].bound.max_x();
                let y = view.elems[SELECTED_RECT].bound.max_y();
                view.elems[CONFIRM_BTN].bound = Rect::new(
                    x - 20.0,
                    y + 20.0f64.copysign(state.screen_height - y - 50.0),
                    x - 120.0,
                    y + 60.0f64.copysign(state.screen_height - y - 50.0),
                );
            }

            background(
                scene,
                view.elems[FULL_SCREEN_OVERLAY].bound,
                Color::rgba8(16, 16, 16, 75),
            );
            if view.elems[SELECTED_RECT].bound.width().abs() >= 50.0
                && view.elems[SELECTED_RECT].bound.height().abs() >= 40.0
            {
                confirm_btn(scene, view.elems[CONFIRM_BTN]);
            }
            area_selection_rect(scene, view.elems[SELECTED_RECT].bound);
        }

        Page::TextExtract => {
            let PageData::TextExtract(ref mut page_data) = state.page_data else {
                return;
            };
            state.redraw = true;

            let screen_rect = Rect::new(0.0, 0.0, state.screen_width, state.screen_height);

            // clear the window for the screen capture
	    if !page_data.window_cleared {
		scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::TRANSPARENT,
                    None,
                    &screen_rect,
		);
		page_data.window_cleared = true;
		return;
	    }

            let extracted_text = EXTRACTED_TEXT.lock().unwrap();
	    if let Some(ref text) = *extracted_text {
		println!("{text}");
		let mut clipboard = Clipboard::new().unwrap();

		#[cfg(target_os = "linux")]
		{
		    let timeout = Instant::now() + Duration::from_millis(500);
		    clipboard.set().wait_until(timeout).text(text.to_owned()).unwrap();
		}

		#[cfg(target_os = "windows")]
		clipboard.set_text(text.to_owned()).unwrap();

		// exit
		std::process::exit(0);
	    } else {
		background(scene, view.elems[0].bound, Color::rgba8(16, 16, 16, 75));
                spinner(scene, screen_rect, page_data.time.elapsed().as_secs_f64());
	    }
        }
    }
}

fn background(scene: &mut Scene, rect: Rect, color: Color) {
    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);
}

fn area_selection_rect(scene: &mut Scene, rect: Rect) {
    let fill_color = Color::rgba8(175, 175, 175, 20);
    let stroke_color = Color::WHITE;
    scene.fill(Fill::EvenOdd, Affine::IDENTITY, fill_color, None, &rect);
    scene.stroke(
        &Stroke::new(3.0).with_dashes(1.0, [15.0, 10.0]),
        Affine::IDENTITY,
        stroke_color,
        None,
        &rect.to_rounded_rect(5.0),
    );
}

fn confirm_btn(scene: &mut Scene, elem: ViewElement) {
    let fill_color = if elem.mouse_enter {
        Color::rgba8(70, 70, 70, 220)
    } else {
        Color::BLACK
    };
    let stroke_color = Color::WHITE;
    let size = 24.0;
    let font = Font::new(Blob::new(Arc::new(ROBOTO_FONT)), 0);
    let font_ref = to_font_ref(&font).unwrap();
    let font_size = vello::skrifa::instance::Size::new(size);
    let charmap = font_ref.charmap();
    let glyph_metrics = font_ref.glyph_metrics(font_size, LocationRef::new(&[]));
    let metrics = font_ref.metrics(font_size, LocationRef::new(&[]));
    let line_height = metrics.ascent - metrics.descent + metrics.leading;

    const TEXT: &str = "OK";
    let mut pen_x = 0f32;
    let mut glyphs = [Glyph::default(); TEXT.len()];

    for (ch, glyph) in TEXT.chars().zip(glyphs.iter_mut()) {
        let gid = charmap.map(ch).unwrap_or_default();
        let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
        let x = pen_x;
        pen_x += advance;
        *glyph = Glyph {
            id: gid.to_u32(),
            x,
            y: 0.0,
        };
    }

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        fill_color,
        None,
        &elem.bound.to_rounded_rect(5.0),
    );

    scene.stroke(
        &Stroke::new(3.0),
        Affine::IDENTITY,
        stroke_color,
        None,
        &elem.bound.to_rounded_rect(5.0),
    );

    scene
        .draw_glyphs(&font)
        .transform(Affine::translate((
            elem.bound.x0 + (elem.bound.width() - pen_x as f64) / 2.0,
            if elem.bound.y0 > elem.bound.y1 {
                elem.bound.y0 + elem.bound.height() + line_height as f64
            } else {
                elem.bound.y0 + line_height as f64
            },
        )))
        .brush(stroke_color)
        .font_size(size)
        .hint(false)
        .draw(Fill::NonZero, glyphs.into_iter())
}

fn spinner(scene: &mut Scene, rect: Rect, time: f64) {
    let spinner_fill_color = Color::WHITE;
    let background_fill_color = Color::BLACK;
    let spinner = CircleSegment::new(rect.center(), 15.0, 11.0, 3.0 * time, 3.0 * PI / 2.0);
    let background = Rect::from_center_size(rect.center(), (80.0, 80.0)).to_rounded_rect(5.0);
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        background_fill_color,
        None,
        &background,
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        spinner_fill_color,
        None,
        &spinner,
    );
}

fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}

fn clamp_width(rect: &mut Rect, min: f64, max: f64) {
    rect.x0 = rect.x0.clamp(min, max);
    rect.x1 = rect.x1.clamp(min, max);
}

fn clamp_height(rect: &mut Rect, min: f64, max: f64) {
    rect.y0 = rect.y0.clamp(min, max);
    rect.y1 = rect.y1.clamp(min, max);
}
