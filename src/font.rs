#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Font {
    name: String,
    size: u8,
}

impl Font {
    pub fn as_canvas_string(&self) -> String {
        format!("{}px {}", self.size, self.name)
    }

    pub fn new(name: &str, size: u8) -> Self {
        Font {
            name: name.to_string(),
            size,
        }
    }
}
