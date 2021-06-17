use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlTexture};

use crate::error::GlyphAtlasError;
use crate::glyph_atlas::GlyphAtlas;
use crate::shader::{compile_shader, link_program};
pub use crate::font::Font;

mod error;
mod glyph_atlas;
mod packing;
pub mod shader;
mod font;

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
    quads: Vec<BlitQuad>,
}

impl<'a> Renderer<'a> {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Renderer, GlyphAtlasError> {
        gl.enable(WebGl2RenderingContext::BLEND);

        let vert_shader = compile_shader(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            include_str!("shader.vert"),
        )?;

        let frag_shader = compile_shader(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            include_str!("shader.frag"),
        )?;

        let program = link_program(gl, &vert_shader, &frag_shader)?;
        let atlas = GlyphAtlas::new();
        let texture = gl.create_texture().ok_or_else(|| {
            GlyphAtlasError::WebGlError("Could not allocate texture.".to_string())
        })?;

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_image_2d_with_u32_and_u32_and_image_data(
            WebGl2RenderingContext::TEXTURE_2D,    // target
            0,                                     // level
            WebGl2RenderingContext::RGBA as i32,   // internalformat
            WebGl2RenderingContext::RGBA,          // format
            WebGl2RenderingContext::UNSIGNED_BYTE, // type
            &atlas.image_data(),                   // data
        )
        .map_err(|_| GlyphAtlasError::WebGlError("Could not write to texture.".to_string()))?;

        let buffer = gl.create_buffer().ok_or_else(|| {
            GlyphAtlasError::WebGlError("Could not create vertex buffer.".to_string())
        })?;
        let quads = Vec::new();

        Ok(Renderer {
            gl,
            program,
            atlas,
            queued_text: Vec::new(),
            texture,
            buffer,
            quads,
        })
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
        self.quads.clear();
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

        let x_scale = 2. / width;
        let y_scale = 2. / height;
        let x_offset = -1.;
        let y_offset = -1.;

        for (text, font, x, y) in self.queued_text.drain(..) {
            let chars: Vec<char> = text.chars().collect();
            let mut x = x;

            for ch in chars {
                let entry = self.atlas.get_entry(ch, &font);

                let (tex_upper_left, tex_lower_right) = entry.texture_scaled_bounds();

                let glyph_width = entry.glyph_shape.glyph_width();
                let glyph_height = entry.glyph_shape.height();
                let glyph_offset = entry.glyph_shape.descent;

                let blit_upper_left = [
                    (x.round() * x_scale) + x_offset,
                    ((y - glyph_offset as f32 + glyph_height as f32) * y_scale) + y_offset,
                ];
                let blit_lower_right = [
                    ((x.round() + glyph_width as f32) * x_scale) + x_offset,
                    ((y - glyph_offset as f32) * y_scale) + y_offset,
                ];

                self.quads.push(BlitQuad::new(
                    blit_lower_right,
                    blit_upper_left,
                    tex_lower_right,
                    tex_upper_left,
                ));

                x += entry.glyph_shape.occupied_width;
            }
        }

        self.gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));

        unsafe {
            let vert_array = js_sys::Float32Array::view(&bytemuck::cast_slice(&self.quads));

            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        }

        BlitVertex::describe(self.gl, &self.program);

        self.gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            (self.quads.len() * 6) as i32,
        );

        self.gl.finish();
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct BlitQuad {
    vertices: [BlitVertex; 6],
}

impl BlitQuad {
    pub fn new(
        upper_left: [f32; 2],
        lower_right: [f32; 2],
        tex_upper_left: [f32; 2],
        tex_lower_right: [f32; 2],
    ) -> BlitQuad {
        let upper_right = [lower_right[0], upper_left[1]];
        let lower_left = [upper_left[0], lower_right[1]];
        let tex_upper_right = [tex_lower_right[0], tex_upper_left[1]];
        let tex_lower_left = [tex_upper_left[0], tex_lower_right[1]];

        BlitQuad {
            vertices: [
                BlitVertex {
                    position: upper_left,
                    tex_coord: tex_upper_left,
                },
                BlitVertex {
                    position: upper_right,
                    tex_coord: tex_upper_right,
                },
                BlitVertex {
                    position: lower_left,
                    tex_coord: tex_lower_left,
                },
                BlitVertex {
                    position: upper_right,
                    tex_coord: tex_upper_right,
                },
                BlitVertex {
                    position: lower_right,
                    tex_coord: tex_lower_right,
                },
                BlitVertex {
                    position: lower_left,
                    tex_coord: tex_lower_left,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct BlitVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
}

impl BlitVertex {
    fn describe(gl: &WebGl2RenderingContext, program: &WebGlProgram) {
        let mut offset = 0;

        // Binding is simple because each attribute happens to be a [f32; 2].
        for attribute in &["a_position", "a_tex_coord"] {
            let location = gl.get_attrib_location(&program, attribute) as u32;
            gl.vertex_attrib_pointer_with_i32(
                location,
                2,
                WebGl2RenderingContext::FLOAT,
                false,
                std::mem::size_of::<BlitVertex>() as i32,
                offset,
            );
            gl.enable_vertex_attrib_array(location);

            offset += std::mem::size_of::<[f32; 2]>() as i32;
        }
    }
}
