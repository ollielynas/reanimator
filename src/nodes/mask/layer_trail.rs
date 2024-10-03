use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, BlitMask, BlitTarget, DrawParameters, Rect, Surface, Texture2d};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{
    node::{random_id, MyNode},
    nodes::node_enum::NodeType,
    storage::Storage,
};

#[derive(Savefile)]
pub struct LayerTrailNode {
    x: f32,
    y: f32,
    id: String,
    #[savefile_versions = "1.."]
    fade: f32,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    last_frame: Option<Texture2d>,
}

impl Default for LayerTrailNode {
    fn default() -> Self {
        LayerTrailNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            last_frame: None,
            fade: 0.9,
        }
    }
}
impl MyNode for LayerTrailNode {
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
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
     

    fn type_(&self) -> NodeType {
        NodeType::LayerTrail
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            LayerTrailNode::savefile_version(),
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

        let fragment_shader_src = include_str!("layer_trail.glsl");

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

        if self.last_frame.is_none()
            || self.last_frame.as_ref().unwrap().width() != texture.width()
            || self.last_frame.as_ref().unwrap().height() != texture.height()
        {
            let new_texture =
                match Texture2d::empty(&storage.display, texture.width(), texture.height()) {
                    Ok(a) => a,
                    Err(_) => return Err(anyhow!("failed to create texture")),
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
                fade: self.fade,
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

            texture2.as_surface().blit_color(
                &Rect {
                    left: 0,
                    bottom: 0,
                    width: texture2.width(),
                    height: texture2.height(),
                },
                &last_frame.as_surface(),
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    width: last_frame.width() as i32,
                    height: last_frame.height() as i32,
                },
                glium::uniforms::MagnifySamplerFilter::Linear,
            );
            return Ok(());
        } else {
            return Err(anyhow!("last frame not found"));
        }
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        if ui.button("reset") {
            self.last_frame = None;
        }

        ui.slider("fade", 0.0, 1.0, &mut self.fade);
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("basic node, for debugging purposes")
    }
}
