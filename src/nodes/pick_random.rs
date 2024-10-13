use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};

use super::node_enum::NodeType;

#[derive(Savefile)]
pub struct RandomInputNode {
    x: f32,
    y: f32,
    id: String,
    weights: Vec<f32>,
}

impl Default for RandomInputNode {
    fn default() -> Self {
        RandomInputNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            weights: vec![1.0, 1.0],
        }
    }
}

impl MyNode for RandomInputNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "msc"]
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
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

     

    fn type_(&self) -> NodeType {
        NodeType::RandomInput
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            RandomInputNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return self
            .weights
            .iter()
            .enumerate()
            .map(|(i, _)| format!("Input {}", i + 1))
            .collect();
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Output".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("randomly picks an input based on a set of user defined weights");
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, _storage: &Storage) {
        ui.text("inputs: ");

        for (i, value) in self.weights.iter_mut().enumerate() {
            ui.input_float(format!("weight {}", i + 1), value).build();
        }
        if ui.button("add weight") {
            self.weights.push(1.0);
        }
        if self.weights.len() > 0 {
            if ui.button("remove weight") {
                self.weights.pop();
            }
        }
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        _renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        if self.weights.len() < 1 {
            return Err(anyhow!("not enough weights"));
        }
        // let num: f32 = fastrand::f32();
        let total1 = self.weights.iter().sum::<f32>() * fastrand::f32();
        let mut total2 = 0.0;
        let mut index = 0;
        for (i, v) in self.weights.iter().enumerate() {
            total2 += v;
            if total2 > total1 {
                index = i;
                break;
            }
        }

        let input_id = self.input_id(&self.inputs()[index]);
        let output_id =self.output_id(&self.outputs()[0]);
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        let fragment_shader_src = r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            void main() {
            color = texture(tex, v_tex_coords);
            }
            "#;

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("cannot find input texture")),
        };

                storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;
        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture from storage")),
        };
let shader = storage
            .get_frag_shader(fragment_shader_src.to_string())
            .unwrap();
        let uniforms = uniform! {
            tex: texture,

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
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
