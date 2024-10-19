use crate::state::*;
use std::f64::consts::PI;
use std::sync::Arc;

use copypasta::{ClipboardContext, ClipboardProvider};

use vello::kurbo::{Affine, CircleSegment, PathEl, Point, Rect, Stroke, TranslateScale};
use vello::peniko::{Blob, Color, Fill, Font, Format::*, Image};
use vello::skrifa::instance::LocationRef;
use vello::skrifa::raw::FontRef;
use vello::skrifa::MetadataProvider;
use vello::{Glyph, Scene};

use winit::window::CursorIcon;

const ROBOTO_FONT: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

pub fn draw(state: &mut AppState, view: &mut View) {
    let scene = &mut view.scene;
    match state.page {
        Page::AreaSelect => {
            let mouse = view.mouse_position;
            let PageData::AreaSelect(ref mut page_data) = *state.page_data else {
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
            let PageData::TextExtract(ref mut page_data) = *state.page_data else {
                return;
            };
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

            // return if the new window not created yet
            if !page_data.window_created {
                return;
            }

            // Get the transformation for the image and scene elements
            let img_width = page_data.rect.width().abs();
            let img_height = page_data.rect.height().abs();
            let image = Image::new(
                page_data.blob.clone(),
                Rgba8,
                img_width as u32,
                img_height as u32,
            );
            let scale = (state.screen_width / img_width).min(state.screen_height / img_height);
            let iw = img_width * scale;
            let ih = img_height * scale;
            let transform = Affine::translate((
                (state.screen_width - iw) / 2.0,
                (state.screen_height - ih) / 2.0,
            )) * Affine::scale(scale);

            background(scene, screen_rect, Color::rgba8(16, 16, 16, 255));
            scene.draw_image(&image, transform);

            if !page_data.extracted {
                state.redraw = true;
                spinner(scene, screen_rect, page_data.time.elapsed().as_secs_f64());
                let mut static_elems = EXTRACTED_ELEMS.lock().unwrap();
                if let Some((ref rects, ref extracted_text)) = *static_elems {
                    view.elems.push(ViewElement {
                        bound: screen_rect,
                        active: true,
                        cursor: CursorIcon::Crosshair,
                        ..Default::default()
                    });
                    for _ in 0..rects.len() {
                        view.elems.push(ViewElement {
                            active: true,
                            cursor: CursorIcon::Text,
                            ..Default::default()
                        });
                    }
                    let mut ctx = ClipboardContext::new().unwrap();
                    ctx.set_contents(extracted_text.to_owned()).unwrap();
                    // page_data.text = extracted_text.to_string();
                    page_data.rotated_rects = rects.to_vec();
                    page_data.extracted = true;
                    *static_elems = None;
                }
                return;
            }
            let fill_color = Color::rgba8(0, 116, 255, 50);
            for (i, rotated_rect) in page_data.rotated_rects.iter().enumerate() {
                let rect = Rect::from(rotated_rect);
                let [scale, _, _, _, trans_x, trans_y] = transform.as_coeffs();
                let trans_scale = TranslateScale::new((trans_x, trans_y).into(), scale);
                let bound = trans_scale * rect;
                // the view elements start with the screen rectangle then the text rectangles
                view.elems[i + 1].bound = bound;
                if view.elems[i + 1].mouse_enter {
                    scene.fill(
                        Fill::NonZero,
                        transform,
                        Color::rgba8(0, 116, 255, 90),
                        None,
                        &rect,
                    );
                }
                scene.fill(Fill::NonZero, transform, fill_color, None, rotated_rect);
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

#[derive(Debug, Default, Clone, Copy)]
pub struct RotatedRect {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

impl RotatedRect {
    fn min_x(&self) -> f64 {
        self.p0.x.min(self.p1.x).min(self.p2.x).min(self.p3.x)
    }

    fn min_y(&self) -> f64 {
        self.p0.y.min(self.p1.y).min(self.p2.y).min(self.p3.y)
    }

    fn max_x(&self) -> f64 {
        self.p0.x.max(self.p1.x).max(self.p2.x).max(self.p3.x)
    }

    fn max_y(&self) -> f64 {
        self.p0.y.max(self.p1.y).max(self.p2.y).max(self.p3.y)
    }
}

pub struct RotatedRectIter {
    pub rect: RotatedRect,
    pub idx: usize,
}

impl Iterator for RotatedRectIter {
    type Item = PathEl;

    fn next(&mut self) -> Option<PathEl> {
        self.idx += 1;
        match self.idx {
            1 => Some(PathEl::MoveTo(self.rect.p0)),
            2 => Some(PathEl::LineTo(self.rect.p1)),
            3 => Some(PathEl::LineTo(self.rect.p2)),
            4 => Some(PathEl::LineTo(self.rect.p3)),
            5 => Some(PathEl::ClosePath),
            _ => None,
        }
    }
}

impl vello::kurbo::Shape for RotatedRect {
    type PathElementsIter<'iter> = RotatedRectIter;

    fn path_elements(&self, _tolerance: f64) -> RotatedRectIter {
        RotatedRectIter {
            rect: *self,
            idx: 0,
        }
    }

    #[inline]
    fn area(&self) -> f64 {
        (self.p0.x - self.p3.x) * (self.p0.y - self.p3.y)
    }

    #[inline]
    fn perimeter(&self, _accuracy: f64) -> f64 {
        2.0 * ((self.p3.x - self.p0.x) + (self.p3.y - self.p0.y))
    }

    #[inline]
    fn winding(&self, pt: Point) -> i32 {
        if pt.x >= self.p0.x && pt.x < self.p3.x && pt.y >= self.p0.y && pt.y < self.p3.y {
            1
        } else {
            0
        }
    }

    #[inline]
    fn bounding_box(&self) -> Rect {
        Rect::from_points(self.p0, self.p3)
    }
}

impl From<rten_imageproc::RotatedRect> for RotatedRect {
    fn from(value: rten_imageproc::RotatedRect) -> RotatedRect {
        let corners = value.corners();
        let mut new_corners = [Point::ZERO; 4];
        for (i, point) in corners.iter().enumerate() {
            new_corners[i] = Point::new(point.x as f64, point.y as f64);
        }
        let [p0, p1, p2, p3] = new_corners;
        RotatedRect { p0, p1, p2, p3 }
    }
}

impl From<&RotatedRect> for Rect {
    fn from(value: &RotatedRect) -> Rect {
        Rect::new(value.min_x(), value.min_y(), value.max_x(), value.max_y())
    }
}
