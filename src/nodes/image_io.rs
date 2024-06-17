
use std::collections::HashMap;

use fastrand;
use imgui::draw_list::Image;
use imgui::TextureId;
use imgui_glium_renderer::Texture;
use savefile::prelude::*;
use crate::nodes::node_enum::*;
use crate::node::{random_id, MyNode, VERSION};
use crate::project::Storage;
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};
fn default_image() -> Vec<u8> {
    return vec![];
    // Image::from_file_with_format(include_bytes!("./generic-image-placeholder.png"), Some(ImageFormat::Png))
}




#[derive(Savefile)]
pub struct OutputNode {
    x: f32,
    y: f32,
    id: String, 
    #[savefile_ignore]
    #[savefile_introspect_ignore ]
    #[savefile_default_fn="default_image"]
    frame: Vec<u8>,
}

impl Default for OutputNode {
    fn default() -> Self {
        OutputNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            frame: default_image(),
        }
    }
}

impl MyNode for OutputNode {

    fn run(&self, storage: &mut Storage, map: HashMap::<String, String>) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };
        if let Some(frame) = storage.get_frame(get_output) {
            
        }else {
            return false
        }
        return true;
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn path(&self) -> Vec<&str> {
        vec!["IO"]
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
    

    fn name(&self) -> String {
        "Output".to_string()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self) -> Result<(), SavefileError> {
        return save_file(format!("./saves/{}/{}.bin",self.type_().name(), self.id()), VERSION, self);
    }
    
    fn edit_menu_render(&self) {
        
    }
    
    fn inputs(&self) -> Vec<String> {
        return vec!["Frame Input".to_string()];
        }
        
        fn outputs(&self) -> Vec<String> {
        return vec![];
    }
}