use std::{
    any::Any,
    collections::HashMap,
    path::PathBuf,
};

use glium::{Texture2d};

use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;

// use typer::TextRenderer;
// use glium_text_rusttype::TextSystem;

use crate::{
    fonts::MyFonts,
    node::{random_id, MyNode},
    nodes::node_enum::NodeType,
    storage::Storage,
};

#[derive(Savefile)]
pub struct TextMaskNode {
    x: f32,
    y: f32,
    id: String,
    font: String,
    font_size: f32,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    font_data: Option<String>,

    text_data: Vec<u8>,
    output_size: (u32, u32),
}

impl Default for TextMaskNode {
    fn default() -> Self {
        TextMaskNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            font: "None".to_string(),
            font_data: Some(String::new()),
            font_size: 35.0,
            text_data: vec![],
            output_size: (1, 1),
        }
    }
}

impl TextMaskNode {
    fn gen_font_data(&mut self, _storage: &Storage, _renderer: &mut Renderer) {
        let fonts = MyFonts::new();

        if self.font != "Default" {
            if let Ok(handle) = fonts.fonts.select_family_by_name(&self.font) {
                let mut font = None;
                for h in handle.fonts() {
                    let mut fonts = vec![];
                    match h.load() {
                        Ok(a) => {
                            // log::info!("{:?}",a.full_name());

                            fonts.push(a);
                        }
                        Err(_) => {}
                    }
                    fonts.sort_by_key(|x| {
                        x.full_name()
                            .to_lowercase()
                            .replace("bold", "bolddddddddddddddd")
                            .replace("regular", "")
                            .len()
                    });
                    if fonts.len() > 0 {
                        font = Some(fonts[0].clone());
                    }
                    log::info!("{font:?}");
                }
                if let Some(_font) = font {
                    if let Some(_data) = &self.font_data {
                        // imgui_glium_renderer::Renderer::render(&mut self, target, draw_data)
                        // reder
                        // let system = glium_text_rusttype::TextSystem::new(&rcc);
                        // renderer.reload_font_texture(ctx)
                    }
                }
            }
        }
    }
}

impl MyNode for TextMaskNode {
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
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
     

    fn type_(&self) -> NodeType {
        NodeType::TextMask
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
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

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        _renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let input_id = self.input_id(&self.inputs()[0]);
        let output_id =self.output_id(&self.outputs()[0]);;
        let _get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        let _text = storage.get_text(&input_id);

        // if let Some(text) = text {
        //     if let Some(font_data) = self.font_data {

        //     }
        // }

        if self.text_data.len() as u32 == self.output_size.0 * self.output_size.1 * 4 {
            storage.create_and_set_texture(
                self.output_size.0,
                self.output_size.1,
                output_id.clone(),
            );
            let _texture: &Texture2d = storage.get_texture(&output_id).unwrap();
            // texture.write(Rect {
            //     bottom: 0,
            //     left: 0,
            //     width: texture.width(),
            //     height: texture.height(),
            // }, Image2d:: self.text_data);
        }

        return Ok(());
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Render text as a mask");
    }
}
