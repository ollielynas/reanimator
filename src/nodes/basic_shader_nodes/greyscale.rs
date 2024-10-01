use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use itertools::Itertools;
use savefile::{save_file, SavefileError};
use anyhow::anyhow;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::nodes::node_enum::NodeType;
use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};

#[derive(Savefile, EnumIter, PartialEq, Eq, Copy, Clone)]
enum GreyscaleType {
    Sum,
    Desaturation,
}

const PRESETS: [(&str, [f32; 3], GreyscaleType); 2] = [
    (
        "ENGGEN 131",
        [1.0, 5.0 / 3.0, 1.0 / 3.0],
        GreyscaleType::Sum,
    ),
    ("Desaturate", [1.0, 1.0, 1.0], GreyscaleType::Desaturation),
];

impl GreyscaleType {
    fn index(&self) -> i32 {
        match self {
            GreyscaleType::Desaturation => 0,
            GreyscaleType::Sum => 1,
        }
    }
}

#[derive(Savefile)]
pub struct GreyScaleNode {
    x: f32,
    y: f32,
    id: String,
    grey_type: GreyscaleType,
    weights: [f32; 3],
}

impl Default for GreyScaleNode {
    fn default() -> Self {
        GreyScaleNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            grey_type: GreyscaleType::Sum,
            weights: [1.0, 5.0 / 3.0, 1.0 / 3.0],
        }
    }
}

impl MyNode for GreyScaleNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Basic Shader"]
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
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
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::Greyscale
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            GreyScaleNode::savefile_version(),
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
        renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let input_id = self.input_id(&self.inputs()[0]);
        let output_id =self.output_id(&self.outputs()[0]);;
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        let fragment_shader_src = include_str!("greyscale.glsl");

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
            weights: self.weights,
            f_type: self.grey_type.index(),
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
        ui.text_wrapped("a greyscale node with several presets to chose from")
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        ui.color_edit3("weights", &mut self.weights);
        let items = GreyscaleType::iter().collect::<Vec<GreyscaleType>>();
        let (mut item_index, _) = items
            .iter()
            .find_position(|x| x == &&self.grey_type)
            .unwrap_or((0, &GreyscaleType::Sum));
        ui.combo("greyscale type", &mut item_index, &items, |x| {
            match x {
                GreyscaleType::Sum => "Sum",
                GreyscaleType::Desaturation => "Desaturation",
            }
            .into()
        });
        self.grey_type = items[item_index];

        if ui.button("pick from preset") {
            ui.open_popup("preset")
        }
        ui.popup("preset", || {
            for (name, scale, greyscale_type) in PRESETS.iter().rev() {
                if ui.button(name) {
                    self.grey_type = greyscale_type.clone();
                    self.weights = scale.clone();
                }
            }
        });
    }
}
