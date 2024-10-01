use core::{hash, panic};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::path::PathBuf;

use fast_smaz::Smaz;
use font_kit::sources::multi::MultiSource;
use glium::index::NoIndices;
use glium::program::Attribute;
use glium::texture::{self, RawImage2d};
use glium::vertex::VertexBufferAny;
use glium::{glutin::surface::WindowSurface, Display, Texture2d};
use glium::{implement_vertex, Program, Surface};
use image::EncodableLayout;
use image::{DynamicImage, ImageBuffer, Rgba};
use imgui::{TreeNodeFlags, Ui};
use std::hash::Hash;

use crate::fonts::MyFonts;
use crate::render_nodes::RenderNodesParams;
use crate::widgets::link_widget;
use crate::{relaunch_program, LOG_TEXT};

const VERTEX_SHADER: &'static str = r#"
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
    text: HashMap<String, String>,
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
    pub error_texture: Texture2d,
    pub fonts: MyFonts,
    pub project_root: PathBuf,
    lock_output_pos: bool,
    pub max_lines_of_text: usize,
    full_messages: bool,
}

impl Storage {
    pub fn new(display: Display<WindowSurface>) -> Storage {
        let error_image = image::load_from_memory(include_bytes!("img/th.jpg"))
            .unwrap_or_else(|x| DynamicImage::new_rgb8(20, 20))
            .flipv()
            .into_rgba8();
        let not_texture = RawImage2d::from_raw_rgba(
            error_image.as_bytes().to_vec(),
            (error_image.width(), error_image.height()),
        );
        // let a: HashMap<Texture2d, String> = HashMap::new();
        let error_texture: Texture2d = Texture2d::new(&display, not_texture).unwrap();

        // let frame2 = storage.create_and_set_texture(frame.height(), frame.width(), output_id).unwrap();
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }
        implement_vertex!(Vertex, position, tex_coords);
        // We've changed our shape to a rectangle so the image isn't distorted.
        let shape = vec![
            Vertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
        ];
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let vertex_buffer: glium::VertexBuffer<Vertex> =
            glium::VertexBuffer::new(&display, &shape).unwrap();

        let mut s = Storage {
            textures: HashMap::new(),
            text: HashMap::new(),
            display,
            unused_textures: HashMap::new(),
            shaders: HashMap::new(),
            time: 0.0,
            indices,
            vertex_buffer: vertex_buffer.into(),
            cached_textures: HashMap::new(),
            hasher: Box::new(DefaultHasher::new()),
            redirect_id_to_cache: HashMap::new(),
            project_name: String::new(),
            show_debug_window: false,
            error_texture,
            fonts: MyFonts::new(),
            project_root: PathBuf::new(),
            max_lines_of_text: 1,
            lock_output_pos: true,
            full_messages: false,
        };
        return s;
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
    pub fn set_text(&mut self, k: String, v: String) {
        self.text.insert(k, v);
    }

    pub fn get_text(&self, k: &String) -> Option<&String> {
        self.text.get(k)
    }
    pub fn get_texture(&self, k: &String) -> Option<&Texture2d> {
        if let Some(k) = self.redirect_id_to_cache.get(k) {
            return self.cached_textures.get(k);
        } else {
            return self.textures.get(k);
        }
    }
    /// does not reset time, resets memory
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
            if let Some(ref mut a) = self
                .unused_textures
                .get_mut(&(texture.width(), texture.height()))
            {
                if a.len() < 20 {
                    a.push(texture);
                }
            } else {
                self.unused_textures
                    .insert((texture.width(), texture.height()), vec![texture]);
            }
        }
    }

    pub fn debug_window(&mut self, ui: &Ui, params: &mut RenderNodesParams) {
        if !self.show_debug_window {
            return;
        }

        let window: Option<imgui::WindowToken> = ui
            .window("debug window")
            .opened(&mut self.show_debug_window)
            .begin();

        if ui.is_window_hovered() {
            params.moving = false;
            params.scale_changed = false;
        }

        ui.columns(2, "debug", true);
        ui.text_wrapped(format!("Project name: {}", self.project_name));
        ui.text_wrapped(format!("shaders: {}", self.shaders.len()));
        ui.text_wrapped(format!("time: {}", self.time));

        if ui.collapsing_header(
            format!("textures: {}", self.textures.len()),
            TreeNodeFlags::empty(),
        ) {
            for (k, v) in &self.textures {
                ui.text_wrapped(format!("{:?}, {:?}", k, v.dimensions()));
            }
        }

        let mut total = 0;
        for (k, v) in &self.unused_textures {
            total += v.len();
        }

        if ui.collapsing_header(
            format!("unused textures: {}/{}", self.unused_textures.len(), total),
            TreeNodeFlags::empty(),
        ) {
            for (k, v) in &self.unused_textures {
                ui.text_wrapped(format!("{:?}, {}", k, v.len()))
            }
        }

        ui.text_wrapped(format!("cached_textures: {}", self.cached_textures.len()));

        if ui.button("crash (panic)") {
            panic!("debug crash {}", line!());
        }
        if ui.button("relaunch (admin)") {
            #[cfg(target_os = "windows")]
            {
                relaunch_program(true, "");
            }
        }

        ui.next_column();

        ui.text("debug log");
        link_widget(ui, "output.log", "output.log");
        ui.checkbox("include log headers", &mut self.full_messages);

        ui.child_window("log")
            .border(true)
            .scroll_bar(true)
            .build(|| {
                if ui.is_window_hovered() {
                    params.moving = false;
                    params.scale_changed = false;
                }
                match LOG_TEXT.lock() {
                    Ok(a2) => {
                        let a = a2.clone();
                        drop(a2);

                        let total = a.len();

                        let mut increased = false;

                        if total > self.max_lines_of_text {
                            ui.text_disabled("loading more logs..  ({}/{} chunks loaded) ");
                            if ui.scroll_y() == 0.0 {
                                self.max_lines_of_text += 1;
                                increased = true;
                            }
                        }

                        let mut index = 0;
                        for text in a.iter().rev().take(self.max_lines_of_text) {
                            if increased && index == self.max_lines_of_text - 1 {
                                ui.set_scroll_here_y();
                            }
                            let s = String::from_utf8(text.smaz_decompress().unwrap_or_default())
                                .unwrap_or_default();
                            if !self.full_messages {
                                for line in s.lines() {
                                    if line.starts_with("[") {
                                        ui.text_wrapped(line.split_once("]").unwrap_or(("", "")).1);
                                    } else {
                                        ui.text_wrapped(line);
                                    }
                                }
                            } else {
                                ui.text_wrapped(s);
                            }

                            index += 1;
                        }
                    }
                    Err(e) => {
                        ui.text_wrapped("cannot get output");
                    }
                }
            });
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
            log::info!(
                "created shader: (vert/frag) {} / {} bytes",
                vert.bytes().len(),
                frag.bytes().len()
            );
            let program = match glium::Program::from_source(&self.display, &(vert), &frag, None) {
                Ok(a) => a,
                Err(a) => {
                    log::info!(
                        "shader_comp_error:--------------- \n\n\n {a:#?}\n\n\n --------------"
                    );
                    return None;
                }
            };
            self.shaders.insert((vert.clone(), frag.clone()), program);
        }
        return self.shaders.get(&(vert, frag));
    }

    pub fn create_and_set_texture(&mut self, width: u32, height: u32, k: String) {
        match self.unused_textures.get_mut(&(width, height)) {
            Some(a) if a.len() > 0 => {
                let texture = a.pop().unwrap();
                texture
                    .as_surface()
                    .clear_color(148.0 / 255.0, 0.0, 211.0 / 255.0, 1.0);
                self.set_texture(k, texture);
            }
            Some(a) => {
                let image: image::ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
                    width,
                    height,
                    [0_u8].repeat((height * width * 4) as usize),
                )
                .unwrap();
                log::info!("{}", a.len());
                let not_texture: RawImage2d<u8> = RawImage2d::from_raw_rgba(
                    image.as_bytes().to_vec(),
                    (image.width(), image.height()),
                );
                let texture: Texture2d = Texture2d::with_mipmaps(
                    &self.display,
                    not_texture,
                    texture::MipmapsOption::AutoGeneratedMipmaps,
                )
                .unwrap();

                self.set_texture(k, texture);
            }
            _ => {
                let image: image::ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
                    width,
                    height,
                    [0_u8].repeat((height * width * 4) as usize),
                )
                .unwrap();
                log::info!("allocated texture: {}x{}", image.width(), image.height());
                let not_texture: RawImage2d<u8> = RawImage2d::from_raw_rgba(
                    image.as_bytes().to_vec(),
                    (image.width(), image.height()),
                );

                let texture: Texture2d = Texture2d::with_mipmaps(
                    &self.display,
                    not_texture,
                    texture::MipmapsOption::NoMipmap,
                )
                .unwrap();
                self.set_texture(k, texture);
            }
        }
    }
}
