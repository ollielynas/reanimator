use std::{any::Any, collections::HashMap, path::PathBuf};
use glium::{uniform, DrawParameters, Surface};
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

pub struct InvertTextureNode {
    x: f32,
    y: f32,
    id: String,
    #[savefile_versions = "1.."]
    #[savefile_default_val = "false"]
    invert_alpha: bool,
}

impl Default for InvertTextureNode {
    fn default() -> Self {
        InvertTextureNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            invert_alpha: false,
        }
    }
}

impl MyNode for InvertTextureNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Basic Shader"]
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

     

    fn type_(&self) -> NodeType {
        NodeType::InvertTexture
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            InvertTextureNode::savefile_version(),
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

    fn savefile_version() -> u32
    where
        Self: Sized,
    {
        1
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

        let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;
        uniform bool alpha;

        void main() {
            color = vec4(1.0) - texture(tex, v_tex_coords);
            if (!alpha) {
            color.a = texture(tex, v_tex_coords).a;
            }
        }
    "#;

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
            alpha: self.invert_alpha,
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

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, _storage: &Storage) {
        ui.checkbox("invert alpha", &mut self.invert_alpha);
    }
}
