use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, BlitMask, BlitTarget, DrawParameters, Rect, Surface, Texture2d};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};

use super::node_enum::NodeType;

#[derive(Savefile)]
pub struct MotionNode {
    x: f32,
    y: f32,
    id: String,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    last_frame: Option<Texture2d>,
}

impl Default for MotionNode {
    fn default() -> Self {
        MotionNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            last_frame: None,
        }
    }
}
impl MyNode for MotionNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Advanced Shader"]
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
        NodeType::Motion
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            MotionNode::savefile_version(),
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
    ) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return false,
        };

        let fragment_shader_src = include_str!("movement.glsl");

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return false,
        };

        storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .unwrap();
        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return false,
        };

        if self.last_frame.is_none()
            || self.last_frame.as_ref().unwrap().width() != texture.width()
            || self.last_frame.as_ref().unwrap().height() != texture.height()
        {
            let new_texture =
                match Texture2d::empty(&storage.display, texture.width(), texture.height()) {
                    Ok(a) => a,
                    Err(_) => return false,
                };
            texture.as_surface().blit_color(
                &Rect {
                    left: 0,
                    bottom: 0,
                    width: texture.width(),
                    height: texture.height(),
                },
                &new_texture.as_surface(),
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    width: new_texture.width() as i32,
                    height: new_texture.height() as i32,
                },
                glium::uniforms::MagnifySamplerFilter::Linear,
            );
            self.last_frame = Some(new_texture);
        }

        if let Some(last_frame) = &self.last_frame {
            let shader = storage
                .get_frag_shader(fragment_shader_src.to_string())
                .unwrap();

            let uniforms = uniform! {
                tex: texture,
                last_tex: last_frame,

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

            if self.last_frame.is_some() {
                texture.as_surface().blit_color(
                    &Rect {
                        left: 0,
                        bottom: 0,
                        width: texture.width(),
                        height: texture.height(),
                    },
                    &self.last_frame.as_ref().unwrap().as_surface(),
                    &BlitTarget {
                        left: 0,
                        bottom: 0,
                        width: self.last_frame.as_ref().unwrap().width() as i32,
                        height: self.last_frame.as_ref().unwrap().height() as i32,
                    },
                    glium::uniforms::MagnifySamplerFilter::Linear,
                );
            }

            return true;
        } else {
            return false;
        }
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("basic node, for debugging purposes")
    }
}
