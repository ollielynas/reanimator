use std::collections::HashMap;

use glium::index::NoIndices;
use glium::texture::{self, RawImage2d};
use glium::vertex::VertexBufferAny;
use glium::{glutin::surface::WindowSurface, Display, Texture2d};
use glium::{implement_vertex, program, Program, Vertex};
use image::{ImageBuffer, ImageFormat, Rgba};

use image::EncodableLayout;

const VERTEX_SHADER:  &'static str = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

pub struct Storage {
    textures: HashMap<String, Texture2d>,
    pub display: Display<WindowSurface>,
    unused_textures: HashMap<(u32, u32), Vec<Texture2d>>,
    shaders: HashMap<(String, String), Program>,
    pub time: f32,
    pub indices: NoIndices,
    pub vertex_buffer: VertexBufferAny,
}

impl Storage {
    pub fn new(display: Display<WindowSurface>) -> Storage {

                    // let frame2 = storage.create_and_set_texture(frame.height(), frame.width(), output_id).unwrap();
                    #[derive(Copy, Clone)]
                    struct Vertex {
                        position: [f32; 2],
                        tex_coords: [f32; 2],
                    }
                    implement_vertex!(Vertex, position, tex_coords);
                    // We've changed our shape to a rectangle so the image isn't distorted.
                    let shape = vec![
                        Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                        Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] },
                        Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                
                        Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                        Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                        Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                    ];
                    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                    let vertex_buffer: glium::VertexBuffer<Vertex> = glium::VertexBuffer::new(&display, &shape).unwrap();
        

        Storage {
            textures: HashMap::new(),
            display,
            unused_textures: HashMap::new(),
            shaders: HashMap::new(),
            time: 0.0,
            indices,
            vertex_buffer:vertex_buffer.into(),
        }
    }

    pub fn set_texture(&mut self, k: String, v: Texture2d) {
        self.textures.insert(k, v);
    }

    pub fn get_texture(&self, k: &String) -> Option<&Texture2d> {
        self.textures.get(k)
    }

    /// shaders are cached
    pub fn gen_frag_shader(&mut self, frag: String) -> Option<&Program> {
        
        self.gen_shader(VERTEX_SHADER.to_string(), frag)
    }

    pub fn get_frag_shader(&self, frag: String) -> Option<&Program> {

        self.get_shader(VERTEX_SHADER.to_string(), frag)
    }

    pub fn get_shader(&self, vert: String, frag: String) -> Option<&Program> {
        return self.shaders.get(&(vert, frag));
    }

    pub fn gen_shader(&mut self, vert: String, frag: String) -> Option<&Program> {
        if !self.shaders.contains_key(&(vert.clone(), frag.clone())) {
            let program = glium::Program::from_source(&self.display, &(vert), &frag, None).unwrap();

            self.shaders.insert((vert.clone(), frag.clone()), program);
        }

        return self.shaders.get(&(vert, frag));
    }

    pub fn create_and_set_texture(
        &mut self,
        height: u32,
        width: u32,
        k: String,
    ) -> Option<&Texture2d> {
        let k2 = k.clone();

        match self.unused_textures.get_mut(&(width, height)) {
            Some(a) if a.len() > 0 => {
                let texture = a.pop().unwrap();
                self.set_texture(k, texture);
            }
            _ => {
                let image: image::ImageBuffer<Rgba<u8>, Vec<u8>> =
                    ImageBuffer::from_raw(width, height, [0_u8].repeat((height * width * 4) as usize))
                        .unwrap();

                let not_texture = RawImage2d::from_raw_rgba(
                    image.as_bytes().to_vec(),
                    (image.width(), image.height()),
                );
                let texture: Texture2d = Texture2d::new(&self.display, not_texture).unwrap();
                self.set_texture(k, texture);
            }
        }
        return self.get_texture(&k2);
    }
}
