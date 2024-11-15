use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{
    node::{random_id, MyNode},
    nodes::node_enum::{NodeType},
    storage::Storage,
};

#[derive(Savefile)]
pub struct BrightnessRangeMaskNode {
    x: f32,
    y: f32,
    id: String,
    low: f32,
    high: f32,
}

impl Default for BrightnessRangeMaskNode {
    fn default() -> Self {
        BrightnessRangeMaskNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            low: 0.25,
            high: 0.75,
        }
    }
}
impl MyNode for BrightnessRangeMaskNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Mask"]
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
        NodeType::BrightnessRangeMask
    }

     

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, _storage: &Storage) {
        ui.slider("low", 0.0, 1.0, &mut self.low);
        ui.slider("high", 0.0, 1.0, &mut self.high);
        let width = ui.item_rect_size()[1];

        for i in 0..100 {
            let a = (i as f32) / 100.0;
            let b = (1.0 + i as f32) / 100.0;
            ui.get_window_draw_list()
                .add_line(
                    [ui.cursor_pos()[0] + a * width, ui.cursor_pos()[0] * 5.0],
                    [ui.cursor_pos()[0] + b * width, ui.cursor_pos()[0] * 5.0],
                    [a, a, a, 1.0],
                )
                .build();
        }
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            BrightnessRangeMaskNode::savefile_version(),
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
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        let fragment_shader_src = include_str!("brightness_range.glsl");

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
            low: self.low,
            high: self.high,
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

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Create a mask based on the brightness value of a pixel. Returns true if the brightness is higher than low and lower than high. If low is higher than high they are swapped and they are inverted.")
    }
}
