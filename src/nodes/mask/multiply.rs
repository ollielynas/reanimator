use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{
    node::{random_id, MyNode},
    nodes::node_enum::NodeType,
    storage::Storage,
};

#[derive(Savefile)]
pub struct MultiplyNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for MultiplyNode {
    fn default() -> Self {
        MultiplyNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for MultiplyNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Combine"]
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn generic_info(&self) -> GenericNodeInfo {
        GenericNodeInfo {
            x: self.x,
            y: self.y,
            type_: self.type_(),
            id: self.id.to_owned(),
        }
    }

    fn savefile_version() -> u32 {
        2
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

     

    fn type_(&self) -> NodeType {
        NodeType::Multiply
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            MultiplyNode::savefile_version(),
            self,
        );
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["Input 1".to_owned(), "Input 2".to_owned()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Output".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        _renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let output_id =self.output_id(&self.outputs()[0]);;
        let get_outputs = self
            .inputs()
            .iter()
            .map(|x| {
                match map.get(&self.input_id(x)) {
                    Some(a) => a,
                    None => "----BLANK----",
                }
                .to_owned()
            })
            .collect::<Vec<String>>();

        if get_outputs.contains(&"----BLANK----".to_string()) {
            return Err(anyhow!("missing input"));
        }

        let fragment_shader_src = r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D a;
            uniform sampler2D b;



            void main() {
            color = texture(a, v_tex_coords) * texture(b, v_tex_coords);
            }
            "#;

        let texture_size: (u32, u32) = match storage.get_texture(&get_outputs[0]) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("failed to get input texture")),
        };

                storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;
        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let a: &glium::Texture2d = match storage.get_texture(&get_outputs[0]) {
            Some(a) => a,
            None => return Err(anyhow!("failed to create output texture")),
        };
        let b: &glium::Texture2d = match storage.get_texture(&get_outputs[1]) {
            Some(a) => a,
            None => return Err(anyhow!("failed to create output texture")),
        };
let shader = storage
            .get_frag_shader(fragment_shader_src.to_string())
            .unwrap();
        let uniforms = uniform! {
            a: a,
            b: b,
        };
        let texture2 = storage.get_texture(&output_id).unwrap();
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

        return Ok(());
    }
}
