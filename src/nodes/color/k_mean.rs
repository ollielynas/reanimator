use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use anyhow::anyhow;


use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

// use okolors::*;
// use crate::nodes::color::k_mean::palette::Srgb;
use glium::texture::RawImage2d;
use okolors::{palette::Srgb, Okolors};

use crate::{
    node::{random_id, MyNode},
    nodes::node_enum::{self, NodeType},
    storage::Storage,
};

#[derive(Savefile)]
pub struct PalletGenNode {
    x: f32,
    y: f32,
    id: String,
    pallet_size: u8,
    lightness_weight: f32,
    sampling_factor: f32,
    map: HashMap<u64, Vec<u8>>,
    frozen_pallet: Option<Vec<u8>>,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = Box::new(DefaultHasher::new());
    t.hash(&mut hasher);
    hasher.finish()
}
impl Default for PalletGenNode {
    fn default() -> Self {
        PalletGenNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            pallet_size: 16,
            lightness_weight: 0.5,
            sampling_factor: 0.25,
            map: HashMap::new(),
            frozen_pallet: None,
        }
    }
}
impl MyNode for PalletGenNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Color"]
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
        NodeType::PalletGen
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            PalletGenNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Pallet".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        let mut p = self.pallet_size as i32;
        ui.input_int("pallet size", &mut p).build();
        ui.input_float("lightness weight", &mut self.lightness_weight)
            .build();
        ui.input_float("sampling_factor", &mut self.sampling_factor)
            .build();

        self.pallet_size = p.clamp(0, 255) as u8;
        if !self.frozen_pallet.is_some() {
            if ui.button("freeze") {
                self.frozen_pallet = Some(vec![]);
            }
        } else {
            if ui.button("un-freeze") {
                self.frozen_pallet = None;
            }
        }
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

        storage.create_and_set_texture(self.pallet_size as _, 1, output_id.clone());

        let input_texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return Err(anyhow!("unable to find input texture")),
        };

        let texture2 = storage.get_texture(&output_id).unwrap();
        let data = input_texture
            .read_to_pixel_buffer()
            .map()
            .iter()
            .map(|x| [x.0, x.1, x.2])
            .collect::<Vec<[u8; 3]>>();

        if let Some(frozen_pallet) = &self.frozen_pallet {
            if frozen_pallet.len() == self.pallet_size as usize * 3 {
                texture2.write(
                    glium::Rect {
                        left: 0,
                        bottom: 0,
                        width: self.pallet_size as _,
                        height: 1,
                    },
                    RawImage2d::<u8>::from_raw_rgb(
                        frozen_pallet.to_vec(),
                        (self.pallet_size as u32, 1),
                    ),
                );
                return Ok(());
            }
        }

        let new_hash = calculate_hash(&(
            &data,
            self.pallet_size,
            (self.lightness_weight.clone() * 1000000.0) as i32,
            (self.sampling_factor.clone() * 1000000.0) as i32,
        ));

        if !self.map.contains_key(&new_hash) {
            let mut image_data = Vec::new();
            // should probably do something about that .cloned
            if let Ok(mut pallet) = Okolors::try_from(
                data.iter()
                    .cloned()
                    .map(|x| Srgb::new(x[0], x[1], x[2]).into())
                    .collect::<Vec<Srgb<u8>>>()
                    .as_slice(),
            ) {
                // pallet
                let p = pallet
                    .lightness_weight(self.lightness_weight)
                    .parallel(true)
                    .palette_size(self.pallet_size)
                    .sampling_factor(self.sampling_factor)
                    .sort_by_frequency(true)
                    .srgb8_palette();

                for i in p {
                    image_data.extend([i.red, i.green, i.blue]);
                }

                self.map.insert(new_hash, image_data);
            }
        }
        if (self.map.get(&new_hash)).unwrap_or(&vec![]).len() == self.pallet_size as usize * 3 {
            if self.frozen_pallet.is_some() {
                self.frozen_pallet = self.map.get(&new_hash).cloned();
            }
            texture2.write(
                glium::Rect {
                    left: 0,
                    bottom: 0,
                    width: self.pallet_size as _,
                    height: 1,
                },
                RawImage2d::<u8>::from_raw_rgb(
                    self.map.get(&new_hash).unwrap().to_vec(),
                    (self.pallet_size as u32, 1),
                ),
            );
        }
        return Ok(());
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Create a mask based on the brightness value of a pixel. Returns true if the brightness is higher than low and lower than high. If low is higher than high they are swapped and they are inverted.")
    }
}
