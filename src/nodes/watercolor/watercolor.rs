use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{texture::RawImage2d, uniform, DrawParameters, Surface, Texture2d};
use imgui_glium_renderer::Renderer;
use lumo::tracer::Texture;
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage};
use image::EncodableLayout;


#[derive(Savefile)]
pub struct WaterColorNode {
    x: f32,
    y: f32,
    id: String,
    scale: f32,
    size: (u32,u32),
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    paper_texture: Option<Texture2d>
}

impl Default for WaterColorNode {
    fn default() -> Self {
        WaterColorNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            scale: 1.0,
            size: (1,1),
            paper_texture: None,
        }
    }
}
impl MyNode for WaterColorNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image","Artistic"]
    }

    
    fn set_id(&mut self, id: String) {
        self.id = id;
    }


    fn savefile_version() -> u32 {0}

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::WaterColor
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            WaterColorNode::savefile_version(),
            self,
        );
    }


    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Out".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }



    fn run(&mut self, storage: &mut Storage, map: HashMap::<String, String>, renderer: &mut Renderer) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };


    if self.paper_texture.is_none() {
        let image = image::load_from_memory_with_format(
            include_bytes!("textured-paper-background-tile-with-bluish-tint.jpg"),
            image::ImageFormat::Jpeg,
        )
        .unwrap().flipv().into_rgba8();


        let not_texture = RawImage2d::from_raw_rgba(image.as_bytes().to_vec(), (image.width(), image.height()));
        // let a: HashMap<Texture2d, String> = HashMap::new();
        let texture: Texture2d = Texture2d::new(&storage.display, not_texture).unwrap();
        self.paper_texture = Some(texture);
    }

    let fragment_shader_src = include_str!("watercolor.glsl");

    let texture_size:(u32, u32) = match storage.get_texture(get_output) {
        Some(a) => {(a.width(), a.height())},
        None => {return false},
    };
    

    storage.gen_frag_shader(fragment_shader_src.to_string()).unwrap();
    storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());
    
    let texture: &glium::Texture2d = match storage.get_texture(get_output) {
        Some(a) => {a},
        None => {return false},
    };

    let shader = storage.get_frag_shader(fragment_shader_src.to_string()).unwrap();

            let uniforms = uniform! {
                tex: texture,
                u_resolution: [texture_size.0 as f32, texture_size.1 as f32],
                paper: (self.paper_texture.as_ref()).unwrap(),
            };
            let texture2 = storage.get_texture(&output_id).unwrap();
            texture2.as_surface().draw(&storage.vertex_buffer, &storage.indices, shader, &uniforms,
                            &DrawParameters {
                                ..Default::default()
                            }
                            ).unwrap();

        return true;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("basic node, for debugging purposes")
    }
}
