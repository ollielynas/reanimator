use std::{any::Any, collections::HashMap, iter::Filter, path::{self, PathBuf}, process::Output};
use std::hash::Hash;
use fastrand;
use glium::{uniform, BlitTarget, DrawParameters, Surface};
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use savefile::prelude::*;

use crate::{
    nodes::{image_io::OutputNode, node_enum::NodeType},
    storage::Storage,
};


pub fn random_id() -> String {
    fastrand::i32(1000..=9999).to_string()
}


pub trait MyNode {

    fn savefile_version() -> u32 where Self: Sized;

    fn as_any(&self) -> &dyn Any;

    fn path(&self) -> Vec<&str>;

    fn type_(&self) -> NodeType;

    fn y(&self) -> f32;
    fn x(&self) -> f32;

    fn name(&self) -> String {
        self.type_().name()
    }
    fn id(&self) -> String;

    // fn set_pos();

    fn inputs(&self) -> Vec<String>;
    fn outputs(&self) -> Vec<String>;

    fn set_xy(&mut self, x: f32, y: f32);

    fn edit_menu_render(&mut self, ui: &Ui, renderer: &mut Renderer) {
        ui.text("this node cannot be edited");
    }
    fn description(&mut self, ui: &Ui) {
        ui.text("this node does not yet have a description");
    }

    /// # Use This
    /// ```
    /// fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
    ///     return save_file(
    ///         path.join(self.name()).join(self.id()+".bin"),
    ///         NodeStruct::savefile_version(),
    ///         self,
    ///     );
    /// }
    /// ```
    fn save(&self, path: PathBuf) -> Result<(), SavefileError>;

    fn input_id(&self, input: String) -> String {
        format!("node-{}-input-{input}", self.id())
    }
    fn output_id(&self, output: String) -> String {
        format!("node-{}-output-{output}", self.id())
    }

    fn run(&mut self, storage: &mut Storage, map: HashMap<String, String>, renderer: &mut Renderer) -> bool;
}

#[derive(Savefile)]
pub struct DebugNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for DebugNode {
    fn default() -> Self {
        DebugNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for DebugNode {
    fn path(&self) -> Vec<&str> {
        vec!["msc"]
    }

    fn savefile_version() -> u32 {0}

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
        NodeType::Debug
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            DebugNode::savefile_version(),
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

        let fragment_shader_src = 
            r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            void main() {
            color = texture(tex, v_tex_coords);
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
