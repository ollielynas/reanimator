use std::{
    any::Any,
    collections::HashMap,
    path::PathBuf,
};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use node_enum::*;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{node::*, nodes::*, storage::Storage};

#[derive(Savefile)]
pub struct HistogramNode {
    x: f32,
    y: f32,
    id: String,
    sample_size: i32,
    histogram: [[u32;3];256]
}


impl Default for HistogramNode {
    fn default() -> Self {
        HistogramNode::new()
    }
}

impl HistogramNode {
    pub fn new() -> HistogramNode {
        HistogramNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            histogram: [[0;3];256],
            sample_size: 3000,
        }
    }
}

impl MyNode for HistogramNode {
    fn path(&self) -> Vec<&str> {
        vec!["Data"]
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

    fn savefile_version() -> u32
    where
        Self: Sized,
    {
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }


    fn type_(&self) -> NodeType {
        NodeType::Histogram
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }


    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, _storage: &Storage) {
        
        ui.input_int("samples", &mut self.sample_size).build();
        
        let aval = ui.content_region_avail();
        let c_pos = ui.cursor_screen_pos();
        let rect_size =   aval[0].min(aval[1]);
        let _ = ui.get_window_draw_list().add_rect(c_pos, [c_pos[0] +  rect_size, c_pos[1] + rect_size], (0.0,0.0,0.0,1.0)).build();
        for (i, value) in self.histogram.iter().enumerate() {
            ui.get_window_draw_list().add_line(
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size], 
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *value[0] as f32 / self.sample_size as f32 * 10.0], 
                (1.0 * i as f32 / 255.0, 0.0, 0.0)
            ).thickness(rect_size / 256.0).build();
            ui.get_window_draw_list().add_line(
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *value[0] as f32 / self.sample_size as f32 * 10.0], 
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *(value[0] + value[1]) as f32 / self.sample_size as f32 * 10.0], 
                (0.0, 1.0 * i as f32 / 255.0, 0.0)
            ).thickness(rect_size / 256.0).build();
            ui.get_window_draw_list().add_line(
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *(value[0] + value[1]) as f32 / self.sample_size as f32 * 10.0], 
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *(value[0] + value[1] + value[2]) as f32 / self.sample_size as f32 * 10.0], 
                (0.0, 0.0, 1.0 * i as f32 / 255.0)
            ).thickness(rect_size / 256.0).build();
        }
    }

    fn render_in_node(&self, ui: &imgui::Ui, ui_scale: f32,  _renderer: &mut Renderer, _params: &mut crate::render_nodes::RenderNodesParams) {
        
        let c_pos = ui.cursor_screen_pos();
        ui.text("\n\n\n\n\n\n\n\n");
        let aval = ui.content_region_avail();
        let rect_size =   aval[0];
        let _ = ui.get_window_draw_list().add_rect(c_pos, [c_pos[0] +  rect_size, c_pos[1] + rect_size], (0.0,0.0,0.0,1.0)).build();
        for (i, value) in self.histogram.iter().enumerate() {
            ui.get_window_draw_list().add_line(
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size], 
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *value[0] as f32 / self.sample_size as f32 * 10.0], 
                (1.0 * i as f32 / 255.0, 0.0, 0.0)
            ).thickness(rect_size / 256.0).build();
            ui.get_window_draw_list().add_line(
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *value[0] as f32 / self.sample_size as f32 * 10.0], 
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *(value[0] + value[1]) as f32 / self.sample_size as f32 * 10.0], 
                (0.0, 1.0 * i as f32 / 255.0, 0.0)
            ).thickness(rect_size / 256.0).build();
            ui.get_window_draw_list().add_line(
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *(value[0] + value[1]) as f32 / self.sample_size as f32 * 10.0], 
                [c_pos[0] + rect_size * i as f32 /  256.0, c_pos[1] +rect_size - rect_size *(value[0] + value[1] + value[2]) as f32 / self.sample_size as f32 * 10.0], 
                (0.0, 0.0, 1.0 * i as f32 / 255.0)
            ).thickness(rect_size / 256.0).build();
        }
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text("show a color histogram");
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            HistogramNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec![];
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
        let output_id = self.id.clone() + "downsized";
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
            color = texture(tex, v_tex_coords);
            }
            "#;

        let texture_size: (u32, u32) = match storage.get_texture(get_output) {
            Some(a) => (a.width(), a.height()),
            None => return Err(anyhow!("cannot find input texture")),
        };

                storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;
        

        let aspect = texture_size.0 as f32 / texture_size.1 as f32;

        storage.create_and_set_texture(
            texture_size.0.min(((self.sample_size as f32).sqrt() * aspect).max(1.0) as u32), 
            texture_size.1.min(((self.sample_size as f32).sqrt() / aspect).max(1.0) as u32), 
            output_id.clone());

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
            )?;
            self.histogram = [[0;3];256];
            for rgba in texture2
            .read_to_pixel_buffer()
            .map()
            .iter() {
                if rgba.3 != 0 {
                    self.histogram[rgba.0 as usize][0] += 1;
                    self.histogram[rgba.1 as usize][1] += 1;
                    self.histogram[rgba.2 as usize][2] += 1;
                }
            }

            

        return Ok(());
    }
}
