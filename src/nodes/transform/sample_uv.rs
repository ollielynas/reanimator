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




#[derive(Savefile)]
pub struct SampleUvNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for SampleUvNode {
    fn default() -> Self {
        SampleUvNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}
impl MyNode for SampleUvNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "UV"]
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
        NodeType::SampleUV
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            SampleUvNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string(), "UV".to_owned()];
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
        let input_id2 = self.input_id(self.inputs()[1].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return false,
        };
        let get_output2 = match map.get(&input_id2) {
            Some(a) => a,
            None => return false,
        };

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return false,
        };

        let fragment_shader_src = 
        r#"

        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;
        uniform sampler2D uvc;
        void main() {
        vec4 uv = texture(uvc, v_tex_coords);
        color = texture2DLod(tex, vec2(uv.r, uv.g), 0.0);
        }
        "#;
            storage.gen_frag_shader(fragment_shader_src.to_string()).unwrap();

        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return false,
        };
        let texture3: &glium::Texture2d = match storage.get_texture(get_output2) {
            Some(a) => a,
            None => return false,
        };


    
    let shader = storage.get_frag_shader(fragment_shader_src.to_string()).unwrap();

    let uniforms = uniform! {
        tex: texture,
        uvc: texture3,

    };
    let texture2 = storage.get_texture(&output_id).unwrap();
    texture2.as_surface().draw(&storage.vertex_buffer, &storage.indices, shader, &uniforms,
                    &DrawParameters {
                        ..Default::default()
                    }
                    ).unwrap();
        return true;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Samples a image based on a UV map (kinda)")
    }
}