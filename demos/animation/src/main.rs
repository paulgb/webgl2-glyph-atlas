use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use webgl2_glyph_atlas::{Font, Renderer};
use web_sys::WebGl2RenderingContext;

#[allow(unused)]
macro_rules! console_log {
    ($($x: expr), +) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(
            format!($($x),+)));
    )
}

struct RenderContext {
    _gl: &'static WebGl2RenderingContext,
    renderer: Renderer<'static>,
    frame: u32,
}

impl RenderContext {
    pub fn new(gl: WebGl2RenderingContext) -> RenderContext {
        let gl = Box::leak(Box::new(gl));
        let renderer = Renderer::new(gl);
        
        RenderContext {
            _gl: gl,
            renderer,
            frame: 0,
        }
    }

    pub fn render(&mut self) {
        let start_time = web_sys::window().unwrap().performance().unwrap().now();

        let f = ((self.frame as f32) % 200.) + 1.;

        //self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
        //self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.renderer.queue_text("Hello world ", &Font::new("Georgia", 40), 40., f);
        self.renderer.draw();
        self.frame += 1;

        let end_time = web_sys::window().unwrap().performance().unwrap().now();
        let delta = end_time - start_time;
        if delta > 5. {
            console_log!("delta: {}", delta);
        }
    }
}

pub fn main() {

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let window = web_sys::window().unwrap();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let gl = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>().unwrap();

    let mut renderer = RenderContext::new(gl);


    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        
        renderer.render();

        let window = web_sys::window().unwrap();
        window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }) as Box<dyn FnMut()>));

    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
}
