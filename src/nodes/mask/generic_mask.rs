use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use node_enum::*;
use savefile::{save_file, SavefileError};
use strum::IntoEnumIterator;

use crate::{node::*, nodes::*, storage::Storage};



fn default_node_type_for_mask() -> NodeType {
    NodeType::BrightnessMask
}

impl NodeType {
    fn generic_mask_index(&self) -> i32 {
        match self {
            
                NodeType::BrightnessMask => 0, 

                _a => {
                    -1
                    // unreachable!("node type: {a:?} has no index")
                }
            
        }
    }
}

#[derive(Savefile)]
pub struct GenericMaskNode {
    #[savefile_default_fn="default_node_type_for_mask"]
    #[savefile_ignore]
    #[savefile_versions="..0"]
    type_: NodeType,
    #[savefile_versions="1.."]
    type_index: i32,
    x: f32,
    y: f32,
    id: String,
    input: f32,
    /// leave the name blank to prevent it from being shown
    input_name: String,
    input_min: f32,
    input_max: f32,
}


impl GenericMaskNode {

    pub fn new(type_: NodeType) -> GenericMaskNode {
        GenericMaskNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            type_,
            type_index: type_.generic_mask_index(),
            input: match type_ {
                NodeType::BrightnessMask => 0.5,
                a => {unreachable!("node type: {a:?} is not a generic shader type or has not has the input default value fully implemented")}
            },
            input_name: match type_ {
                NodeType::BrightnessMask => "Threshold".to_owned(),
                a => {unreachable!("node type: {a:?} is not a generic shader type or has not has the input name fully implemented")}
            },
            input_min: match type_ {
                NodeType::BrightnessMask => 0.0,
                a => {unreachable!("node type: {a:?} is not a generic shader type or has not has the min value fully implemented")}
            },
            input_max: match type_ {
                NodeType::BrightnessMask => 1.0,
                a => {unreachable!("node type: {a:?} is not a generic shader type or has not has the max value fully implemented")}
            },
        }
    }
}


impl MyNode for GenericMaskNode {

    fn path(&self) -> Vec<&str> {
        
        vec!["Image","Mask"]
    }


    
    fn set_id(&mut self, id: String) {
        self.id = id;
    }


    fn savefile_version() -> u32 where Self: Sized {
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
        self.type_
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui , _renderer: &mut Renderer, storage: &Storage) {
        if self.input_name.is_empty() {
            ui.text("This shader has no inputs");
            return;
        }

        if self.input_max == f32::MAX || self.input_min == f32::MIN {
            ui.input_float(&self.input_name, &mut self.input).build();
        }else{
            ui.slider(&self.input_name, self.input_min, self.input_max, &mut self.input);
        }

    }


    fn description(&mut self, ui: &imgui::Ui) {
        match self.type_ {
            NodeType::BrightnessMask => {
                ui.text("Blur Image");
            },
            a => {unreachable!("node type: {a:?} is not a generic shader type or has not has the max value fully implemented")}
        };
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            GenericMaskNode::savefile_version(),
            self,
        );
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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



    fn run(&mut self, storage: &mut Storage, map: HashMap::<String, String>, _renderer: &mut Renderer) -> bool {

        if self.type_index != self.type_.generic_mask_index() {
            println!("{}", self.type_index);
            for i in NodeType::iter() {
                if i.generic_mask_index() == self.type_index {
                    // self.type_ = i;
                    let new = GenericMaskNode::new(i);
                    self.type_ = new.type_;
                    self.input_max = new.input_max;
                    self.input_name = new.input_name;
                    println!("{:?}", self.type_);
                    break
                }
            }
        }

        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };

        let fragment_shader_src = 
            match self.type_ {
            NodeType::BrightnessMask => include_str!("brightness.glsl"),
            a => {unreachable!("node type: {a:?} is not a generic shader type or has not has the input default value fully implemented")}
        };

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
                u_time: storage.time,
                u_input: self.input,
                u_resolution: [texture_size.0 as f32, texture_size.1 as f32],
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
