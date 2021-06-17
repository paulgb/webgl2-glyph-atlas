use web_sys::{Document, Window};

use crate::error::{GlyphAtlasError, Result};

pub fn get_window() -> Result<Window> {
    web_sys::window()
        .ok_or_else(|| GlyphAtlasError::DomError("Could not access window global.".to_string()))
}

pub fn get_document() -> Result<Document> {
    get_window()?
        .document()
        .ok_or_else(|| GlyphAtlasError::DomError("Cloud not access document.".to_string()))
}
