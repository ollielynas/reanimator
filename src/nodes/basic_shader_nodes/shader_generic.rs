use std::{
    any::Any,
    collections::HashMap,
    ops::{RangeBounds, RangeInclusive},
    path::PathBuf,
};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use node_enum::*;
use savefile::{save_file, SavefileError};
use strum::IntoEnumIterator;

use crate::{node::*, nodes::*, storage::Storage};


impl NodeType {
    fn generic_shader_index(&self) -> i32 {
        match self {
            
                NodeType::ChromaticAberration => 0, 
                NodeType::VHS => 1,
                NodeType::Blur => 2,
                NodeType::Dot => 3,
                NodeType::Sharpness => 4,
                a => {
                    -1
                    // unreachable!("node type: {a:?} has no index")
                }
            
        }
    }
}

fn default_node_type() -> NodeType {
    NodeType::Blur
}

#[derive(Savefile)]
pub struct GenericShaderNode {
    #[savefile_default_fn="default_node_type"]
    #[savefile_ignore]
    #[savefile_versions="..0"]
    type_: NodeType,
    #[savefile_versions="1.."]
    type_index: i32,
    x: f32,
    y: f32,
    id: String,
    input: f32,
    /// leave the name blank to prevent it from being shown
    input_name: String,
    input_min: f32,
    input_max: f32,
}

impl GenericShaderNode {
    pub fn new(type_: NodeType) -> GenericShaderNode {
        println!("{:?}", type_);
        GenericShaderNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            type_index: type_.generic_shader_index(),
            type_,
            input: match type_ {
                NodeType::ChromaticAberration 
                | NodeType::VHS => 0.5,
                NodeType::Blur => 10.0,
                NodeType::Dot => 3.0,
                NodeType::Sharpness => 1.0,
                a => {
                    unreachable!("node type: {a:?} is not a generic shader type or has not has the input default value fully implemented")
                }
            },
            input_name: match type_ {
                NodeType::ChromaticAberration | NodeType::VHS => "Strength".to_owned(),
                NodeType::Blur => "Radius".to_owned(),
                NodeType::Dot => "Radius".to_owned(),
                NodeType::Sharpness => "Sharpness".to_owned(),
                a => {
                    unreachable!("node type: {a:?} is not a generic shader type or has not has the input name fully implemented")
                }
            },
            input_min: match type_ {
                NodeType::ChromaticAberration => f32::MIN,
                NodeType::VHS => 0.0,
                NodeType::Blur => 0.0,
                NodeType::Dot => 0.001,
                NodeType::Sharpness => 0.0,
                a => {
                    unreachable!("node type: {a:?} is not a generic shader type or has not has the min value fully implemented")
                }
            },
            input_max: match type_ {
                NodeType::ChromaticAberration => f32::MAX,
                NodeType::VHS => 1.0,
                NodeType::Dot => 20.0,
                NodeType::Blur => f32::MAX,
                NodeType::Sharpness => 4.0,
                a => {
                    unreachable!("node type: {a:?} is not a generic shader type or has not has the max value fully implemented")
                }
            },
        }
    }
}

impl MyNode for GenericShaderNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Basic Shader"]
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn savefile_version() -> u32
    where
        Self: Sized,
    {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        self.type_
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer) {
        if self.input_name.is_empty() {
            ui.text("This shader has no inputs");
            return;
        }

        if self.input_max == f32::MAX || self.input_min == f32::MIN {
            ui.input_float(&self.input_name, &mut self.input).build();
        } else {
            ui.slider(
                &self.input_name,
                self.input_min,
                self.input_max,
                &mut self.input,
            );
        }
    }

    fn description(&mut self, ui: &imgui::Ui) {
        match self.type_ {
            NodeType::ChromaticAberration => {
                // ui.set_window_font_scale(2.0);
                let begin_pos = ui.cursor_pos();
                ui.text_colored([0.0, 0.0, 1.0, 0.5], "Chromatic Aberration");
                ui.set_cursor_pos([begin_pos[0] + 1.0, begin_pos[1]]);
                ui.text_colored([0.0, 1.0, 0.0, 0.5], "Chromatic Aberration");
                ui.set_cursor_pos([begin_pos[0] + 2.0, begin_pos[1]]);
                ui.text_colored([1.0, 0.0, 0.0, 0.5], "Chromatic Aberration");
                // ui.set_window_font_scale(1.0);
            }
            NodeType::VHS => {
                ui.text_wrapped("Warning! This not is not currently working");
            }
            NodeType::Blur => {
                ui.text_wrapped("Blur Image using Gaussian blur");
            }
            NodeType::Dot => {
                ui.text_wrapped("Dots\nTODO: better description");
            }
            NodeType::Sharpness => {
                ui.text_wrapped("Sharpens Image");
            }
            a => {
                unreachable!("node type: {a:?} is not a generic shader type or has not has the max value fully implemented")
            }
        };
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            GenericShaderNode::savefile_version(),
            self,
        );
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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



        if self.type_index != self.type_.generic_shader_index() {
            println!("{}", self.type_index);
            for i in NodeType::iter() {
                if i.generic_shader_index() == self.type_index {
                    // self.type_ = i;
                    let new = GenericShaderNode::new(i);
                    self.type_ = new.type_;
                    self.input_max = new.input_max;
                    self.input_name = new.input_name;
                    println!("{:?}", self.type_);
                    break
                }
            }
        }

        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return false,
        };

        let fragment_shader_src = match self.type_ {
            NodeType::ChromaticAberration => {
                r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            uniform float u_time;
            uniform float u_input;
            uniform vec2 u_resolution;

            uniform vec2 r_displacement = vec2(3.0, 0.0);
            uniform vec2 g_displacement = vec2(0.0, 0.0);
            uniform vec2 b_displacement = vec2(-3.0, 0.0);

            void main() {
            float r = texture(tex, v_tex_coords + (u_input * r_displacement) / u_resolution).r;
            float g = texture(tex, v_tex_coords + (u_input * g_displacement) / u_resolution).g;
            float b = texture(tex, v_tex_coords + (u_input * b_displacement) / u_resolution).b;
            color = vec4(r,g,b, texture(tex, v_tex_coords).a);
            }
            "#
            }
            NodeType::VHS => include_str!("VHS.glsl"),
            NodeType::Blur => include_str!("gaussian.glsl"),
            NodeType::Dot => include_str!("dot.glsl"),
            NodeType::Sharpness => include_str!("sharp.glsl"),
            a => {
                unreachable!("node type: {a:?} is not a generic shader type or has not has the input default value fully implemented")
            }
        };

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

        let shader = storage
            .get_frag_shader(fragment_shader_src.to_string())
            .unwrap();

        let uniforms = uniform! {
            tex: texture,
            u_time: storage.time as f32,
            u_input: self.input,
            u_resolution: [texture_size.0 as f32, texture_size.1 as f32],
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
                    // time_elapsed_query: Some(),
                    ..Default::default()
                },
            )
            .unwrap();

        return true;
    }
}
