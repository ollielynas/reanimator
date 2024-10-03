


use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{texture::RawImage2d, uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{
    node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage
};


#[derive(Savefile)]
pub struct LogicNotNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for LogicNotNode {
    fn default() -> Self {
        LogicNotNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for LogicNotNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Mask", "Logic"]
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
        NodeType::LogicNot
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            LogicNotNode::savefile_version(),
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

        let fragment_shader_src = r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            void main() {
            color = vec4(1.0) - texture(tex, v_tex_coords);
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
        ui.text_wrapped("Logical Not (1 - value)")
    }
}
#[derive(Savefile)]
pub struct LogicAndNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for LogicAndNode {
    fn default() -> Self {
        LogicAndNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}

impl MyNode for LogicAndNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Mask", "Logic"]
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
        NodeType::LogicAnd
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            LogicNotNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["Input 1".to_string(), "Input 2".to_string()];
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
        let output_id =self.output_id(&self.outputs()[0]);;
        let input_id = self.input_id(&self.inputs()[0]);
        let input_id2 = self.input_id(&self.inputs()[1]);
        let get_output1 = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input 1")),
        };
        let get_output2: &String = match map.get(&input_id2) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input 2")),
        };

        let fragment_shader_src = r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            uniform sampler2D tex2;
            void main() {
            color = (texture(tex2, v_tex_coords) * texture(tex, v_tex_coords));
            }
            "#;

        let texture_size: (u32, u32) = match storage.get_texture(get_output1) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("cannot find input texture")),
        };

                storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;
        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(get_output1) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture 1 from storage")),
        };
        let texture2: &glium::Texture2d = match storage.get_texture(get_output2) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture 2 from storage")),
        };
let shader = storage
            .get_frag_shader(fragment_shader_src.to_string())
            .unwrap();
        let uniforms = uniform! {
            tex: texture,
            tex2: texture2,

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
        ui.text_wrapped("Logical And (a*b)")
    }
}
#[derive(Savefile)]
pub struct LogicOrNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for LogicOrNode {
    fn default() -> Self {
        LogicOrNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for LogicOrNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Mask", "Logic"]
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
        NodeType::LogicOr
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            LogicOrNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["Input 1".to_string(), "Input 2".to_string()];
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
        let output_id = self.output_id(&self.outputs()[0]);
        let input_id = self.input_id(&self.inputs()[0]);
        let input_id2 = self.input_id(&self.inputs()[1]);
        let get_output1 = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input 1")),
        };
        let get_output2: &String = match map.get(&input_id2) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input 2")),
        };

        let fragment_shader_src = r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            uniform sampler2D tex2;
            void main() {
            color = max(texture(tex2, v_tex_coords), texture(tex, v_tex_coords));
            }
            "#;

        let texture_size: (u32, u32) = match storage.get_texture(get_output1) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("cannot find input texture")),
        };

                storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;
        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(get_output1) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture 1 from storage")),
        };
        let texture2: &glium::Texture2d = match storage.get_texture(get_output2) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture 2 from storage")),
        };
let shader = storage
            .get_frag_shader(fragment_shader_src.to_string())
            .unwrap();
        let uniforms = uniform! {
            tex: texture,
            tex2: texture2,

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
        ui.text_wrapped("Logical And (min(a,b))")
    }
}
