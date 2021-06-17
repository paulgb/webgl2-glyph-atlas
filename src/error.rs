pub enum GlyphAtlasError {
    WebGlError(String),
    WebGlShaderInfoLog(String),
    WebGlProgramInfoLog(String),
}

impl std::fmt::Display for GlyphAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error")
    }
}

impl std::fmt::Debug for GlyphAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}