use std::{any::Any, collections::HashMap, path::PathBuf};

use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, nodes::node_enum, storage::Storage};

use node_enum::NodeType;


#[derive(Savefile)]
pub struct TextInputNode {
    x: f32,
    y: f32,
    id: String,
    text: String,
}

impl Default for TextInputNode {
    fn default() -> Self {
        TextInputNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            text: String::new(),
        }
    }
}
impl MyNode for TextInputNode {
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
        NodeType::TextInput
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            TextInputNode::savefile_version(),
            self,
        );
    }


    fn inputs(&self) -> Vec<String> {
        return vec![];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Out".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }


    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, _: &Storage) {
        let region = ui.content_region_avail();
        ui.input_text_multiline("Text", &mut self.text, region).build();
    }



    fn run(&mut self, storage: &mut Storage, _map: HashMap::<String, String>, _renderer: &mut Renderer) -> bool {
        let output_id = self.output_id(self.outputs()[0].clone());


        storage.set_text(output_id, self.text.clone());

        return true;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("basic node, for debugging purposes")
    }
}
