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
pub struct ColorNoiseNode {
    x: f32,
    y: f32,
    id: String,
    input: bool,
    size: (u32, u32),
    #[savefile_versions = "1.."]
    #[savefile_default_val = "1"]
    seed: i32,
}

impl Default for ColorNoiseNode {
    fn default() -> Self {
        ColorNoiseNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            input: false,
            size: (1, 1),
            seed: 1,
        }
    }
}
impl MyNode for ColorNoiseNode {
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

     

    fn type_(&self) -> NodeType {
        NodeType::ColorNoise
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            ColorNoiseNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        if self.input {
            return vec!["In".to_string()];
        } else {
            return vec![];
        }
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
        let output_id =self.output_id(&self.outputs()[0]);;

        let fragment_shader_src = r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;
            
            uniform sampler2D tex;
            uniform float time;
            uniform float seed; 
            
            float random (vec2 st) {
            return fract(sin(dot(st.xy,
                                vec2(12.9898,78.233)))*
                43758.5453123);
            }



            void main() {

            float rnd_r = random( v_tex_coords * (time + seed) * seed);
            float rnd_g = random( v_tex_coords * ((time+seed) * seed ) );
            float rnd_b = random( v_tex_coords * (time * seed*2.3 ) );

            color = vec4(rnd_r, rnd_g, rnd_b, 1.0);
            }
            "#;

        if self.input && self.inputs().len() > 0 {
            let input_id = self.input_id(&self.inputs()[0]);
            let get_output = match map.get(&input_id) {
                Some(a) => a,
                None => return Err(anyhow!("missing input")),
            };
            self.size = match storage.get_texture(get_output) {
                Some(a) => (a.width(), a.height()),
                None => return Err(anyhow!("missing input")),
            };
        }

                storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;
        storage.create_and_set_texture(self.size.0, self.size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(&output_id) {
            Some(a) => a,
            None => {
                return Err(anyhow!("failed to get texture"));
            }
        };
let shader = storage
            .get_frag_shader(fragment_shader_src.to_string())
            .unwrap();
        let uniforms = uniform! {
            tex: texture,
            time: storage.time as f32,
            seed: (10.0/( self.seed % 100) as f32),

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
        ui.text_wrapped("source of white noise");
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, _storage: &Storage) {
        ui.checkbox("use input texture for dimensions", &mut self.input);
        ui.disabled(self.input, || {
            let mut input_val = [self.size.0 as i32, self.size.1 as i32];
            ui.input_int2("dimensions (w,h)", &mut input_val).build();
            self.size = (input_val[0].max(1) as u32, input_val[1].max(1) as u32);
        });
        ui.input_int("seed", &mut self.seed).build();
        if ui.button("gen random seed") {
            self.seed = fastrand::i32(i32::MIN..i32::MAX);
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
