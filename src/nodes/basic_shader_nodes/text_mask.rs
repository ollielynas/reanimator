use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage};



#[derive(Savefile)]
pub struct TextMaskNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for TextMaskNode {
    fn default() -> Self {
        TextMaskNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for TextMaskNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image","msc"]
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
        NodeType::TextMask
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            TextMaskNode::savefile_version(),
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

        

        

        let text = storage.get_text(&input_id);

        


    storage.create_and_set_texture(10, 10, output_id.clone());
        



            let texture2 = storage.get_texture(&output_id).unwrap();
            

        return true;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("basic node, for debugging purposes")
    }
}
