use std::{collections::HashMap, fs, path::PathBuf};

use glium::{implement_vertex, texture, uniform, DrawParameters, Frame, Surface};
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage};


#[derive(Savefile)]
pub struct InvertTextureNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for InvertTextureNode {
    fn default() -> Self {
        InvertTextureNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}


impl MyNode for InvertTextureNode {
    fn path(&self) -> Vec<&str> {
        vec!["msc"]
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::InvertTexture
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            0,
            self,
        );
    }

    fn edit_menu_render(&self) {}

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



    fn run(&self, storage: &mut Storage, map: HashMap::<String, String>) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };

        let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = vec4(1.0) - texture(tex, v_tex_coords);
            color.a = texture(tex, v_tex_coords).a;
        }
    "#;


    let texture_size:(u32, u32) = match storage.get_texture(get_output) {
        Some(a) => {(a.height(), a.width())},
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
            };
            let texture2 = storage.get_texture(&output_id).unwrap();
            texture2.as_surface().draw(&storage.vertex_buffer, &storage.indices, shader, &uniforms,
                            &DrawParameters {
                                ..Default::default()
                            }
                            ).unwrap();

        

        

        return true;
    }
}