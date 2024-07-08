use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};

use super::node_enum::NodeType;

#[derive(Savefile)]
pub struct SplitRgbaNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for SplitRgbaNode {
    fn default() -> Self {
        SplitRgbaNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for SplitRgbaNode {
    fn path(&self) -> Vec<&str> {
        vec!["RGBA"]
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
        NodeType::SplitRgba
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            SplitRgbaNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec![
            "Red".to_string(),
            "Green".to_string(),
            "Blue".to_string(),
            "Alpha".to_string(),
            ];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
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

        let fragment_shader_src = r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            uniform int rgba_index;

            void main() {
                
            float channel = 0.0;
            vec4 px = texture(tex, v_tex_coords);
            if (rgba_index == 0) {
                channel = px.r;
                }else if (rgba_index == 1) {
                channel = px.g;
                }else if (rgba_index == 2) {
                channel = px.b;
                }else if (rgba_index == 3) {
                channel = px.a;
            }

            color = vec4(channel);
            }
            "#;

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.height(), a.width()),
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
                rgba_index: i as i32,
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
