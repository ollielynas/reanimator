use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{texture::RawImage2d, uniform, BlitTarget, DrawParameters, Rect, Surface, Texture2d};
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
pub struct DelayNode {
    x: f32,
    y: f32,
    id: String,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    frames: Vec<Texture2d>,
    frame_delay_count: i32,
}

impl Default for DelayNode {
    fn default() -> Self {
        DelayNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            frames: vec![],
            frame_delay_count: 1,
        }
    }
}
impl MyNode for DelayNode {
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

     

    fn type_(&self) -> NodeType {
        NodeType::Delay
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            DelayNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["Input".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Delayed Output".to_string()];
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

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("cannot find input texture")),
        };

        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let input_texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return Err(anyhow!("unable to find input texture")),
        };

        if self.frame_delay_count != self.frames.len() as i32 {
            self.frames.clear();

            for _ in 0..self.frame_delay_count {
                let frame = RawImage2d::from_raw_rgb(
                    vec![
                        0;
                        (3 * input_texture.dimensions().0 * input_texture.dimensions().1) as usize
                    ],
                    input_texture.dimensions(),
                );

                let texture2 = match Texture2d::new(&storage.display, frame) {
                    Ok(a) => a,
                    Err(e) => {
                        log::info!("{e:?}");
                        return Ok(());
                    }
                };

                input_texture.as_surface().blit_color(
                    &Rect {
                        left: 0,
                        bottom: 0,
                        width: input_texture.width(),
                        height: input_texture.height(),
                    },
                    &texture2.as_surface(),
                    &BlitTarget {
                        left: 0,
                        bottom: 0,
                        width: texture2.width() as i32,
                        height: texture2.height() as i32,
                    },
                    glium::uniforms::MagnifySamplerFilter::Linear,
                );

                self.frames.push(texture2);
            }
        }

        if let Some(delayed_frame) = self.frames.pop() {
            let texture3 = match storage.get_texture(&output_id) {
                Some(a) => a,
                None => {
                    // log::info!("no texture found: {}", output_id);
                    return Err(anyhow!("no previous frames to display"))
                }
            };
            delayed_frame.as_surface().blit_color(
                &Rect {
                    left: 0,
                    bottom: 0,
                    width: delayed_frame.width(),
                    height: delayed_frame.height(),
                },
                &texture3.as_surface(),
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    width: texture3.width() as i32,
                    height: texture3.height() as i32,
                },
                glium::uniforms::MagnifySamplerFilter::Linear,
            );

            input_texture.as_surface().blit_color(
                &Rect {
                    left: 0,
                    bottom: 0,
                    width: input_texture.width(),
                    height: input_texture.height(),
                },
                &delayed_frame.as_surface(),
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    width: delayed_frame.width() as i32,
                    height: delayed_frame.height() as i32,
                },
                glium::uniforms::MagnifySamplerFilter::Linear,
            );

            self.frames.insert(0, delayed_frame);
        }

        return Ok(());
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Delays the output by a given numebr of frames");
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        ui.input_int("frame delay count", &mut self.frame_delay_count)
            .allow_tab_input(true)
            .build();
        self.frame_delay_count = self.frame_delay_count.clamp(1, 10);
    }
}
