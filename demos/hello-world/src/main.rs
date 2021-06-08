use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext};

use webgl2_glyph_atlas::{Font, Renderer};

pub fn main() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let mut renderer = Renderer::new(&gl);

    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    renderer.render_text("Hello world...", &Font::new("Georgia", 10), 40, 120);
    renderer.render_text("Hello, world! 😮", &Font::new("Arial", 30), 10, 10);
    renderer.render_text("Hello, world", &Font::new("Helvetica", 43), 10, 200);

    Ok(())
}
