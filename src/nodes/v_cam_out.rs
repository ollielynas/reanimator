use std::{any::Any, collections::HashMap, path::PathBuf};


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
pub struct VCamNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for VCamNode {
    fn default() -> Self {
        VCamNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for VCamNode {
    fn path(&self) -> Vec<&str> {
        vec!["IO"]
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
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
     

    fn type_(&self) -> NodeType {
        NodeType::Debug
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            VCamNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec![];
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
        let input_id = self.input_id(&self.inputs()[0]);
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        let _texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("cannot find input texture")),
        };

        let texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture from storage")),
        };

        let _px: glium::pixel_buffer::PixelBuffer<(u8, u8, u8, u8)> = texture.read_to_pixel_buffer();

        return Ok(());
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("register virtual camera")
    }
}
