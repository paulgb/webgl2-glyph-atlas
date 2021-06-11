use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlTexture, WebGlBuffer};

use crate::glyph_atlas::GlyphAtlas;
use crate::shader::{compile_shader, link_program};

mod glyph_atlas;
mod packing;
pub mod shader;

#[allow(unused)]
macro_rules! console_log {
    ($($x: expr), +) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(
            format!($($x),+)));
    )
}

pub struct Renderer<'a> {
    gl: &'a WebGl2RenderingContext,
    program: WebGlProgram,
    atlas: GlyphAtlas,
    queued_text: Vec<(String, Font, f32, f32)>, // TODO: Use FontIndex, not Font
    texture: WebGlTexture,
    buffer: WebGlBuffer,
}

impl<'a> Renderer<'a> {
    pub fn new(gl: &WebGl2RenderingContext) -> Renderer {
        gl.enable(WebGl2RenderingContext::BLEND);

        let vert_shader = compile_shader(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            include_str!("shader.vert"),
        )
        .unwrap();

        let frag_shader = compile_shader(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            include_str!("shader.frag"),
        )
        .unwrap();

        let program = link_program(gl, &vert_shader, &frag_shader).unwrap();
        let atlas = GlyphAtlas::new();
        let texture = gl.create_texture().unwrap();

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_image_2d_with_u32_and_u32_and_image_data(
            WebGl2RenderingContext::TEXTURE_2D,    // target
            0,                                     // level
            WebGl2RenderingContext::RGBA as i32,   // internalformat
            WebGl2RenderingContext::RGBA,          // format
            WebGl2RenderingContext::UNSIGNED_BYTE, // type
            &atlas.image_data(),                   // data
        ).unwrap();

        let buffer = gl.create_buffer().unwrap();

        Renderer {
            gl,
            program,
            atlas,
            queued_text: Vec::new(),
            texture,
            buffer,
        }
    }

    fn bind_texture(&mut self) {
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));

        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
    }

    pub fn queue_text(&mut self, text: &str, font: &Font, x: f32, y: f32) {
        self.queued_text
            .push((text.to_string(), font.clone(), x, y));
    }

    pub fn draw(&mut self) {
        let width = self.gl.drawing_buffer_width() as f32;
        let height = self.gl.drawing_buffer_height() as f32;

        self.gl.use_program(Some(&self.program));
        self.bind_texture();

        let need_to_update_texture = self.atlas.prepare_text(
            self.queued_text
                .iter()
                .map(|(text, font, _, _)| (text.as_str(), font))
                .collect(),
        );

        if need_to_update_texture {
            self.gl
                .tex_sub_image_2d_with_u32_and_u32_and_image_data(
                    WebGl2RenderingContext::TEXTURE_2D,    // target
                    0,                                     // level
                    0,                                     // xoffset
                    0,                                     // yoffset
                    WebGl2RenderingContext::RGBA,          // format
                    WebGl2RenderingContext::UNSIGNED_BYTE, // type
                    &self.atlas.image_data(),              // data
                )
                .unwrap();
        }

        let mut blits: Vec<BlitArea> = Vec::new(); // TODO: use with_capacity

        let x_scale = 2. / width;
        let y_scale = 2. / height;
        let x_offset = -1.;
        let y_offset = -1.;

        for (text, font, x, y) in self.queued_text.drain(..) {
            let chars: Vec<char> = text.chars().collect();
            let mut x: f32 = x as f32;

            for ch in chars {
                let entry = self.atlas.get_entry(ch, &font);

                let (tex_upper_left, tex_lower_right) = entry.texture_scaled_bounds();

                let glyph_width = entry.glyph_shape.glyph_width();
                let glyph_height = entry.glyph_shape.height();
                let glyph_offset = entry.glyph_shape.descent;

                let blit_upper_left = [
                    (x.round() as f32 * x_scale) + x_offset,
                    ((y - glyph_offset as f32 + glyph_height as f32) * y_scale) + y_offset,
                ];
                let blit_lower_right = [
                    ((x.round() + glyph_width as f32) * x_scale) + x_offset,
                    ((y - glyph_offset as f32) * y_scale) + y_offset,
                ];

                blits.push(BlitArea {
                    lower_right: blit_lower_right,
                    upper_left: blit_upper_left,
                    tex_upper_left,
                    tex_lower_right,
                });

                x += entry.glyph_shape.occupied_width;
            }
        }

        self.gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));


            unsafe {
                let vert_array = js_sys::Float32Array::view(&bytemuck::cast_slice(&blits));

                self.gl.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    &vert_array,
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                );
            }


        BlitArea::describe(self.gl, &self.program);


        self.gl
            .draw_arrays_instanced(WebGl2RenderingContext::TRIANGLES, 0, 6, 12);

        self.gl.finish();
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct BlitArea {
    pub upper_left: [f32; 2],
    pub lower_right: [f32; 2],
    pub tex_upper_left: [f32; 2],
    pub tex_lower_right: [f32; 2],
}

impl BlitArea {
    fn describe(gl: &WebGl2RenderingContext, program: &WebGlProgram) {
        let mut offset = 0;

        // Binding is simple because each attribute happens to be a [f32; 2].
        for attribute in &[
            "a_upper_left",
            "a_lower_right",
            "a_tex_upper_left",
            "a_tex_lower_right",
        ] {
            let location = gl.get_attrib_location(&program, attribute) as u32;
            gl.vertex_attrib_pointer_with_i32(
                location,
                2,
                WebGl2RenderingContext::FLOAT,
                false,
                std::mem::size_of::<BlitArea>() as i32,
                offset,
            );
            gl.enable_vertex_attrib_array(location);
            gl.vertex_attrib_divisor(location, 1);

            offset += std::mem::size_of::<[f32; 2]>() as i32;
        }
    }
}

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
