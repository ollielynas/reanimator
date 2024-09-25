use std::{any::Any, collections::HashMap, fs::FileType, path::PathBuf};

use glium::{
    uniform, uniforms::MagnifySamplerFilter, Blend, BlitTarget, DrawParameters, Rect, Surface,
};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage
};


#[derive(Savefile, EnumIter, PartialEq, Copy, Clone)]
enum MyFilterType {
    Linear,
    Nearest,
}

impl MyFilterType {
    fn name(&self) -> String {
        match self {
            MyFilterType::Linear => "Linear",
            MyFilterType::Nearest => "Nearest Neighbor",
        }
        .to_owned()
    }

    fn to_magnify_sample_filter(&self) -> MagnifySamplerFilter {
        match self {
            MyFilterType::Linear => MagnifySamplerFilter::Linear,
            MyFilterType::Nearest => MagnifySamplerFilter::Nearest,
        }
    }
}


#[derive(Savefile)]
pub struct ScaleNode {
    x: f32,
    y: f32,
    id: String,
    use_percent: bool,
    target_size: (u32, u32),
    og_size: (u32, u32),
    target_percent: f32,
    filter: MyFilterType,
}

impl Default for ScaleNode {
    fn default() -> Self {
        ScaleNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            use_percent: true,
            target_size: (1, 1),
            og_size: (1, 1),
            target_percent: 100.0,
            filter: MyFilterType::Linear,
        }
    }
}
impl MyNode for ScaleNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Transform"]
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
        NodeType::Scale
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            ScaleNode::savefile_version(),
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

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        ui.text(format!("original size: {:?}", self.og_size));
        ui.checkbox("use percent based scaling", &mut self.use_percent);

        let filter_types = MyFilterType::iter().collect::<Vec<MyFilterType>>();

        let (mut index, _) = MyFilterType::iter().enumerate().find(|x| x.1 == self.filter).unwrap_or((0,MyFilterType::Linear));

        ui.combo("##", &mut index,  &filter_types, |x| {x.name().into()});

        self.filter = filter_types[index];

        ui.disabled(!self.use_percent, || {
            ui.input_float("resize percent", &mut self.target_percent).build();
            if self.use_percent {
                // round evan to annoy anatol 
                self.target_size = (
                    (self.og_size.0 as f32 * self.target_percent / 100.0)
                        .round_ties_even()
                        .max(1.0) as u32,
                    (self.og_size.1 as f32 * self.target_percent / 100.0)
                        .round_ties_even()
                        .max(1.0) as u32,
                );
            }
        });

        ui.disabled(self.use_percent, || {
            let mut target_size = [self.target_size.0 as i32, self.target_size.1 as i32];
            ui.input_int2("target size", &mut target_size).build();
            self.target_size = (target_size[0].max(0) as u32, target_size[1].max(0) as u32)
        });
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        renderer: &mut Renderer,
    ) -> bool {

        if self.use_percent {
            // round evan to annoy anatol 
            self.target_size = (
                (self.og_size.0 as f32 * self.target_percent / 100.0)
                    .round_ties_even()
                    .max(1.0) as u32,
                (self.og_size.1 as f32 * self.target_percent / 100.0)
                    .round_ties_even()
                    .max(1.0) as u32,
            );
        }

        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return false,
        };

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return false,
        };

        self.og_size = texture_size;

        storage.create_and_set_texture(self.target_size.0, self.target_size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return false,
        };

        let texture2 = storage.get_texture(&output_id).unwrap();

        texture.as_surface().blit_color(
            &Rect {
                left: 0,
                bottom: 0,
                height: texture.height(),
                width: texture.width(),
            },
            &texture2.as_surface(),
            &BlitTarget {
                left: 0,
                bottom: 0,
                height: texture2.height() as i32,
                width: texture2.width() as i32,
            },
            self.filter.to_magnify_sample_filter(),
        );

        return true;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("sclaes an image")
    }
}
