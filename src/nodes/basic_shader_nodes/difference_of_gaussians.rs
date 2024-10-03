use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;

use crate::{
    node::{random_id, MyNode}, nodes::node_enum::NodeType, storage::Storage, widgets::link_widget
};




/// https://youtu.be/5EuYKEvugLU?si=EMuCD_k6mjnqy74c
#[derive(Savefile)]
pub struct DifferenceofGaussiansNode {
    x: f32,
    y: f32,
    id: String,
    radius: f32,
    radius_diff: f32,
    scale_on_2nd: f32,
    greyscale: bool,
    do_threshold: bool,
    threshold: f32,
    hyperbole: f32,
    sigma: f32,
}

impl Default for DifferenceofGaussiansNode {
    fn default() -> Self {
        DifferenceofGaussiansNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            radius: 1.4,
            radius_diff: 30.0,
            scale_on_2nd: 0.5,
            greyscale: false,
            threshold: 0.1,
            hyperbole: 1.0,
            do_threshold: true,
            sigma: 10.0,
        }
    }
}
impl MyNode for DifferenceofGaussiansNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image", "Advanced Shader"]
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
        NodeType::DifferenceOfGaussians
    }


    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            DifferenceofGaussiansNode::savefile_version(),
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
        _renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let input_id = self.input_id(&self.inputs()[0]);
        let output_id =self.output_id(&self.outputs()[0]);;
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        let fragment_shader_src = include_str!("diff_gaussian.glsl");

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("size mismatch when fetching texture")),
        };

                storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;
        storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());

        let texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture from storage")),
        };

        let shader = match storage
            .get_frag_shader(fragment_shader_src.to_string()) {
                Some(a) => a,
                None => return Err(anyhow!("failed to find shader")),
            };

        let uniforms = uniform! {
            tex: texture,
            r1: self.radius,
            r2: self.radius + self.radius_diff,
            u_resolution: [texture_size.0 as f32, texture_size.1 as f32],
            weight: self.scale_on_2nd,
            sigma: self.sigma,
            do_threshold: self.do_threshold,
            greyscale: self.greyscale,
            threshold: self.threshold,
        };
        let texture2 = storage.get_texture(&output_id).ok_or(anyhow!("failed to find texture"))?;
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
        ui.text_wrapped("Attempts to recreate the effects made in this youtube video:");
        ui.new_line();
        link_widget(
            ui,
            "video by acerola".to_owned(),
            "https://youtu.be/5EuYKEvugLU?si=EMuCD_k6mjnqy74c".to_owned(),
        );
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, storage: &Storage) {
        ui.input_float("Inner Radius", &mut self.radius).build();
        ui.input_float("Radius Difference", &mut self.radius_diff)
            .build();
        ui.slider("Weight", 0.0, 1.0, &mut self.scale_on_2nd);
        ui.checkbox("greyscale", &mut self.greyscale);

        ui.checkbox("Enable Threshold", &mut self.do_threshold);
        ui.disabled(!self.do_threshold, || {
            ui.slider("Threshold", 0.0, 1.0, &mut self.threshold);
            ui.input_float("sigma", &mut self.sigma).build();
        });
    }
}
