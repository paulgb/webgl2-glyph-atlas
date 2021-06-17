use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

use webgl2_glyph_atlas::{log_error, Font, Renderer};

pub fn main() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let mut renderer = Renderer::new(&gl).unwrap();
    gl.blend_func(
        WebGl2RenderingContext::ONE,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    renderer.queue_text("Hello world", &Font::new("Georgia", 40), 40., 200.);
    renderer.queue_text("Hello world", &Font::new("Georgia", 45), 40., 200.);
    renderer.queue_text("Hello world", &Font::new("Georgia", 50), 40., 200.);
    renderer.queue_text("Hello world", &Font::new("Georgia", 55), 40., 200.);
    renderer.queue_text("Hello world", &Font::new("Georgia", 60), 40., 200.);
    renderer.queue_text("Hello world", &Font::new("Georgia", 65), 40., 200.);
    renderer.queue_text("Hello world", &Font::new("Georgia", 70), 40., 200.);
    renderer.queue_text("Hello world", &Font::new("Georgia", 75), 40., 200.);

    let _ = renderer.draw().map_err(log_error);

    Ok(())
}
