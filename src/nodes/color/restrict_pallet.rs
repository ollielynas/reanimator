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
use anyhow::anyhow;


use crate::{node::*, nodes::*, storage::Storage};

#[derive(Savefile)]
pub struct RestrictPalletNode {
    x: f32,
    y: f32,
    id: String,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl RestrictPalletNode {
    pub fn new() -> RestrictPalletNode {
        RestrictPalletNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        }
    }
}

impl MyNode for RestrictPalletNode {
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
        0
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
        NodeType::RestrictPalletRGBA
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        // ui.color_edit3_config(label, value)
        let mut color = [self.red, self.green, self.blue, self.alpha];
        ui.color_edit4_config("restrictions", &mut color)
            .display_mode(imgui::ColorEditDisplayMode::Rgb)
            .alpha_bar(true)
            .hdr(true)
            .input_mode(imgui::ColorEditInputMode::Rgb)
            .options(true)
            .format(imgui::ColorFormat::Float)
            .picker(true)
            .build();

        self.red = color[0];
        self.green = color[1];
        self.blue = color[2];
        self.alpha = color[3];
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text("restrict the red green and blue data to cause color banding");
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            RestrictPalletNode::savefile_version(),
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
            uniform float r;
            uniform float g;
            uniform float b;
            uniform float a;


            void main() {
            float r2 = round(texture(tex, v_tex_coords).r * 255.0 * r * r)/(255.0*r*r);
            float g2 = round(texture(tex, v_tex_coords).g * 255.0 * g * g)/(255.0*g*g);
            float b2 = round(texture(tex, v_tex_coords).b * 255.0 * b * b)/(255.0*b*b);
            float a2 = round(texture(tex, v_tex_coords).a * 255.0 * a * a)/(255.0*a*a);
            color = vec4(r2,g2,b2,1.0);
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
            r: self.red,
            g: self.green,
            b: self.blue,

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
            )?;

        return Ok(());
    }
}
