use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, FontFace};

use webgl2_glyph_atlas::{Font, Renderer};

async fn load_font(family: &str, source: &str) {
    let font_promise = FontFace::new_with_str(family, source).unwrap();
    let result = wasm_bindgen_futures::JsFuture::from(font_promise.load().unwrap()).await.unwrap();
    let font = result.dyn_into::<FontFace>().unwrap();

    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().add(&font).unwrap();
}

async fn draw_text() {
    load_font("Pacifico", "url('Pacifico-Regular.ttf')").await;

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let gl = canvas
        .get_context("webgl2").unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>().unwrap();

    let mut renderer = Renderer::new(&gl);

    gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);
    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    renderer.queue_text("Hello world ", &Font::new("Pacifico", 40), 10., 120.);

    renderer.draw();
}

pub fn main() {
    wasm_bindgen_futures::spawn_local(draw_text());
}
