use std::collections::HashMap;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Document, ImageData, TextMetrics, Window};

const TEXTURE_SIZE: u32 = 256;
use crate::error::{GlyphAtlasError, Result};
use crate::packing::{PackingNode, RectSize};
use crate::Font;

pub type FontIndex = usize;

#[derive(Eq, PartialEq, Hash)]
struct GlyphSpec(pub char, pub FontIndex);

pub struct AtlasEntry {
    upper_left: [u32; 2],
    pub glyph_shape: GlyphShape,
}

impl AtlasEntry {
    pub fn texture_scaled_bounds(&self) -> ([f32; 2], [f32; 2]) {
        let upper_left = [
            self.upper_left[0] as f32 / TEXTURE_SIZE as f32,
            (self.upper_left[1] as f32 + 0.5) / TEXTURE_SIZE as f32,
        ];

        let lower_right = [
            (self.upper_left[0] + self.glyph_shape.glyph_width()) as f32 / TEXTURE_SIZE as f32,
            ((self.upper_left[1] + self.glyph_shape.height()) as f32 + 0.5) / TEXTURE_SIZE as f32,
        ];

        (upper_left, lower_right)
    }
}

pub struct GlyphAtlas {
    packing: PackingNode,
    canvas_context: CanvasRenderingContext2d,
    font_to_index: HashMap<Font, FontIndex>,
    fonts: Vec<Font>, // TODO: ugh
    characters: HashMap<GlyphSpec, AtlasEntry>,
    needed: HashMap<GlyphSpec, GlyphShape>,
}

fn get_window() -> Result<Window> {
    web_sys::window().ok_or_else(|| GlyphAtlasError::DomError("Could not access window global.".to_string()))
}

fn get_document() -> Result<Document> {
    get_window()?.document().ok_or_else(|| GlyphAtlasError::DomError("Cloud not access document.".to_string()))
}

impl GlyphAtlas {
    pub fn new() -> Result<GlyphAtlas> {
        let document = get_document()?;

        let canvas = document
            .create_element("canvas")
            .map_err(|_| GlyphAtlasError::DomError("Could not construct canvas element.".to_string()))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| GlyphAtlasError::DomError("Could not cast canvas element.".to_string()))?;

        canvas
            .set_attribute("width", &TEXTURE_SIZE.to_string())
            .map_err(|_| GlyphAtlasError::DomError("Could not set attribute.".to_string()))?;
        canvas
            .set_attribute("height", &TEXTURE_SIZE.to_string())
            .map_err(|_| GlyphAtlasError::DomError("Could not set attribute.".to_string()))?;

        // For debugging. TODO: feature-gate this?
        /*
        canvas
            .set_attribute(
                "style",
                "height: 512px; width: 512px; image-rendering: pixelated",
            )
            .unwrap();
        document.body().unwrap().append_child(&canvas).unwrap();
         */

        let canvas_context = canvas
            .get_context("2d")
            .map_err(|_| GlyphAtlasError::DomError("Could not get context.".to_string()))?
            .ok_or_else(|| GlyphAtlasError::DomError("A non-2D context has already been requested from this canvas.".to_string()))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| GlyphAtlasError::DomError("Could not cast canvas context.".to_string()))?;

        let packing = PackingNode::new(TEXTURE_SIZE as u32, TEXTURE_SIZE as u32);

        Ok(GlyphAtlas {
            canvas_context,
            packing,
            characters: HashMap::default(),
            font_to_index: HashMap::new(),
            fonts: Vec::new(),
            needed: Default::default(),
        })
    }

    pub fn image_data(&self) -> Result<ImageData> {
        self.canvas_context
            .get_image_data(0., 0., TEXTURE_SIZE as f64, TEXTURE_SIZE as f64)
            .map_err(|_| GlyphAtlasError::DomError("Could not get image data from canvas context.".to_string()))
    }

    fn font_to_index(&mut self, font: &Font) -> usize {
        if let Some(index) = self.font_to_index.get(&font) {
            *index
        } else {
            let len = self.fonts.len();
            self.fonts.push(font.clone());
            self.font_to_index.insert(font.clone(), len);

            len
        }
    }

    pub fn prepare_text(&mut self, strings: Vec<(&str, &Font)>) -> Result<bool> {
        self.needed.clear();
        for (text, font) in strings {
            self.canvas_context.set_font(&font.as_canvas_string());

            let font_idx = self.font_to_index(font);

            for ch in text.chars() {
                let key = GlyphSpec(ch, font_idx);
                if !self.characters.contains_key(&key) && !self.needed.contains_key(&key) {
                    let st: String = ch.to_string();
                    let metrics: TextMetrics = self.canvas_context.measure_text(&st).map_err(|_| GlyphAtlasError::DomError("Could not measure text.".to_string()))?;

                    let glyph_shape = GlyphShape::from_text_metrics(&metrics);

                    self.needed.insert(key, glyph_shape);
                }
            }
        }

        if self.needed.len() == 0 {
            return Ok(false);
        }

        let mut needed: Vec<(GlyphSpec, GlyphShape)> = self.needed.drain().collect();

        needed.sort_by(|(_, s1), (_, s2)| s2.size().area().cmp(&s1.size().area()));

        for (GlyphSpec(ch, font_id), glyph_shape) in needed.into_iter() {
            let size = glyph_shape.size();

            let (x, y) = self.packing.insert_rect(size)
                .ok_or_else(|| GlyphAtlasError::InternalError("Ran out of space to pack rect.".to_string()))?;

            // For debugging. TODO: feature-gate this?
            /*
            self.canvas_context
                .set_stroke_style(&wasm_bindgen::JsValue::from("#ff00ff"));
            self.canvas_context.stroke_rect(x as f64 + 0.5, y as f64 + 0.5, size.width as f64 - 1.0, size.height as f64 - 1.0);
                */

            self.canvas_context.save();

            self.canvas_context
                .set_font(&self.fonts[font_id].as_canvas_string());

            self.canvas_context
                .rect(x as f64, y as f64, size.width as f64, size.height as f64);
            self.canvas_context.clip();

            self.canvas_context
                .fill_text(&ch.to_string(), x as f64, (y + glyph_shape.ascent) as f64)
                .map_err(|_| GlyphAtlasError::DomError("Could render text to canvas context.".to_string()))?;

            self.canvas_context.restore();

            self.characters.insert(
                GlyphSpec(ch, font_id),
                AtlasEntry {
                    glyph_shape,
                    upper_left: [x, y],
                },
            );
        }

        Ok(true)
    }

    pub fn get_entry(&self, c: char, font: &Font) -> Result<&AtlasEntry> {
        let font_idx: FontIndex = *self.font_to_index.get(font).ok_or_else(|| GlyphAtlasError::InternalError("Attempted to render font that is not in index.".to_string()))?;
        let ch = self.characters.get(&GlyphSpec(c, font_idx)).ok_or_else(|| GlyphAtlasError::InternalError("Attempted to render glyph that is not in index.".to_string()))?;

        Ok(ch)
    }
}

#[derive(Debug)]
pub struct GlyphShape {
    pub left: u32,
    pub right: u32,
    pub ascent: u32,
    pub descent: u32,
    pub occupied_width: f32,
}

impl GlyphShape {
    pub fn from_text_metrics(metrics: &TextMetrics) -> GlyphShape {
        let left: f64 = metrics.actual_bounding_box_left();
        let right: f64 = metrics.actual_bounding_box_right();
        let ascent: f64 = metrics.actual_bounding_box_ascent();
        let descent: f64 = metrics.actual_bounding_box_descent();

        GlyphShape {
            left: (-left).ceil() as u32,
            right: right.ceil() as u32,
            ascent: ascent.ceil() as u32 + 1, // TODO: figure out why this is necessary.
            descent: descent.ceil() as u32,
            occupied_width: metrics.width() as f32,
        }
    }

    pub fn glyph_width(&self) -> u32 {
        self.left + self.right
    }

    pub fn height(&self) -> u32 {
        self.ascent + self.descent
    }

    pub fn size(&self) -> RectSize {
        RectSize {
            width: self.glyph_width(),
            height: self.height(),
        }
    }
}
