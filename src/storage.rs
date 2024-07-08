use core::hash;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};

use glium::index::NoIndices;
use glium::texture::{self, RawImage2d};
use glium::vertex::VertexBufferAny;
use glium::{glutin::surface::WindowSurface, Display, Texture2d};
use glium::{implement_vertex, pixel_buffer, program, Program, Vertex};
use image::{ImageBuffer, ImageFormat, Rgba};
use imgui::{TreeNodeFlags, Ui};
use imgui_glium_renderer::Renderer;
use std::hash::Hash;
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
    pub time: f64,
    pub indices: NoIndices,
    pub vertex_buffer: VertexBufferAny,
    cached_textures: HashMap<u64, Texture2d>,
    pub hasher: Box<dyn Hasher>,
    redirect_id_to_cache: HashMap<String, u64>,
    pub project_name: String,
    pub show_debug_window: bool,
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
            cached_textures: HashMap::new(),
            hasher: Box::new(DefaultHasher::new()),
            redirect_id_to_cache: HashMap::new(),
            project_name: String::new(),
            show_debug_window: false,
        }
    }

    pub fn calculate_hash<T: Hash>(&mut self, t: &T) -> u64 {
        t.hash(&mut self.hasher);
        self.hasher.finish()
    }

    /// returns the id of the cached texture (it's hash)
    /// to use this cached texture it needs to have an id assigned to it.
    /// this is done through the function: `set_id_of_cached_texture`
    /// calls to the assigned id will be redirected to the cached texture.
    /// the cache is not deleted each frame but the id redirect table is.
    pub fn cache_texture(&mut self, texture: Texture2d) -> u64 {
        let pixel_buffer = &texture.read_to_pixel_buffer().read().unwrap().to_vec();
        let cache = self.calculate_hash(pixel_buffer);
        self.cached_textures.insert(cache, texture);
        return cache;
    }


    /// this allows a cached texture to be refracted with a texture id string.
    /// this will be reset every frame.
    pub fn set_id_of_cached_texture(&mut self, cached_texture_hash: u64, id: String) {
        self.redirect_id_to_cache.insert(id, cached_texture_hash);
    }

    pub fn cached_texture_exists(&self, hash: u64) -> bool {
        self.cached_textures.contains_key(&hash)
    }

    pub fn set_texture(&mut self, k: String, v: Texture2d) {
        self.textures.insert(k, v);
    }

    pub fn get_texture(&self, k: &String) -> Option<&Texture2d> {
        if let Some(k) = self.redirect_id_to_cache.get(k) {
            return self.cached_textures.get(k);
        }else {
            return self.textures.get(k);
        }
    }

    pub fn reset(&mut self) {
        let mut keys: Vec<String> = vec![];
        for (string, _) in self.textures.iter() {
            keys.push(string.clone())
        }
        // let keys = self.textures.keys().collect::<Vec<&String>>();
        self.redirect_id_to_cache = HashMap::new();
        for key in keys {
            self.drop_texture(key.to_string());
        }
    }

    pub fn drop_texture(&mut self, id: String) {
        let texture = self.textures.remove(&id);

        if let Some(texture) = texture {
            if let Some(ref mut a) = self.unused_textures.get_mut(&(texture.height(), texture.width())) {
                if a.len() < 20 {
                a.push(texture);
            }
            }else {
                self.unused_textures.insert((texture.height(), texture.width()), vec![texture]);
            }
        }
    }


    pub fn debug_window(&mut self, ui: &Ui) {
        if !self.show_debug_window {return}

        let window = ui.window("debug window")
        .opened(&mut self.show_debug_window)
        .begin();

    ui.text_wrapped(format!("Project name: {}", self.project_name));
    ui.text_wrapped(format!("shaders: {}", self.shaders.len()));
    ui.text_wrapped(format!("time: {}", self.time));
    ui.text_wrapped(format!("textures: {}", self.textures.len()));

    let mut total = 0;
    for (k,v) in &self.unused_textures {
        total += v.len();
    }


    if ui.collapsing_header(format!("unused textures: {}/{}", self.unused_textures.len(), total), TreeNodeFlags::BULLET) {
        for (k,v) in &self.unused_textures {
            ui.text_wrapped(format!("{:?}, {}", k, v.len()))
        }
    }
    
        ui.text_wrapped(format!("cached_textures: {}", self.cached_textures.len()));

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
            let program = match glium::Program::from_source(&self.display, &(vert), &frag, None) {
                Ok(a) => a,
                Err(a) => {println!("shader_comp_error: \n {a:?}"); return None;}
            };

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

                let not_texture: RawImage2d<u8> = RawImage2d::from_raw_rgba(
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
