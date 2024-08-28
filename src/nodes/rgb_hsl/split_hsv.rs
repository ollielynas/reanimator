use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{
    node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage
};


#[derive(Savefile)]
pub struct SplitHsvNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for SplitHsvNode {
    fn default() -> Self {
        SplitHsvNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for SplitHsvNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image","RGBA"]
    }

    
    fn set_id(&mut self, id: String) {
        self.id = id;
    }


    fn savefile_version() -> u32 {
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::SplitHsv
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            SplitHsvNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec![
            "Hue".to_string(),
            "Saturation".to_string(),
            "Brightness".to_string(),
            "Alpha".to_string(),
            ];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        renderer: &mut Renderer,
    ) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let output_ids = self.outputs().iter().map(|x| self.output_id(x.to_string())).collect::<Vec<String>>();
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return false,
        };
        
        let fragment_shader_src = include_str!("split_hsv.glsl");

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return false,
        };

        storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .unwrap();
        for i in 0..4 {
        storage.create_and_set_texture(texture_size.0, texture_size.1, output_ids[i].clone());
        }
        
        let input_texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return false,
        };
        
        let shader = storage
        .get_frag_shader(fragment_shader_src.to_string())
        .unwrap();
    for i in 0..4 {
            let uniforms = uniform! {
                tex: input_texture,
                hsva_index: i as i32,
            };
            let texture2 = storage.get_texture(&output_ids[i]).unwrap();
            texture2
                .as_surface()
                .draw(
                    &storage.vertex_buffer,
                    &storage.indices,
                    shader,
                    &uniforms,
                    &DrawParameters {
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        return true;
    }
}
