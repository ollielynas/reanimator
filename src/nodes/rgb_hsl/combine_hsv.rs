use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage};



#[derive(Savefile)]
pub struct CombineHsvNode {
    x: f32,
    y: f32,
    id: String,
    #[savefile_versions="1.."]
    raw: bool,
}

impl Default for CombineHsvNode {
    fn default() -> Self {
        CombineHsvNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            raw: false,
        }
    }
}
impl MyNode for CombineHsvNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image","Color"]
    }


    fn set_id(&mut self, id: String) {
        self.id = id;
    }


    fn savefile_version() -> u32 {1}

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
        NodeType::CombineHsv
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            CombineHsvNode::savefile_version(),
            self,
        );
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Creates an image by combining Hue, Saturation, and Value (brightness)")
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        ui.checkbox("use raw/greyscale inputs", &mut self.raw);
    }


    fn inputs(&self) -> Vec<String> {
        return vec![
            "Hue".to_string(),
            "Saturation".to_string(),
            "Brightness".to_string(),
            "Alpha".to_string(),
        ];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Output".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }



    fn run(&mut self, storage: &mut Storage, map: HashMap::<String, String>, _renderer: &mut Renderer) -> bool {
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_outputs = self.inputs().iter().map(|x| match map.get(&self.input_id(x.to_string())) {
            Some(a) => a,
            None => {"----BLANK----"},
        }.to_owned()).collect::<Vec<String>>();

        if get_outputs.contains(&"----BLANK----".to_string()) {
            return false;
        }

        let fragment_shader_src = include_str!("combine_hsv.glsl");

    let texture_size:(u32, u32) = match storage.get_texture(&get_outputs[0]) {
        Some(a) => {(a.width(), a.height())},
        None => {return false},
    };

    storage.gen_frag_shader(fragment_shader_src.to_string()).unwrap();
    storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());
    
    let texture_h: &glium::Texture2d = match storage.get_texture(&get_outputs[0]) {
        Some(a) => {a},
        None => {return false},
    };
    let texture_s: &glium::Texture2d = match storage.get_texture(&get_outputs[1]) {
        Some(a) => {a},
        None => {return false},
    };
    let texture_v: &glium::Texture2d = match storage.get_texture(&get_outputs[2]) {
        Some(a) => {a},
        None => {return false},
    };
    let texture_a: &glium::Texture2d = match storage.get_texture(&get_outputs[3]) {
        Some(a) => {a},
        None => {return false},
    };

    let shader = storage.get_frag_shader(fragment_shader_src.to_string()).unwrap();

            let uniforms = uniform! {
                tex_h: texture_h,
                tex_s: texture_s,
                tex_v: texture_v,
                tex_a: texture_a,
                raw: self.raw,
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
