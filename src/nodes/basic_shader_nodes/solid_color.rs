use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, nodes::{node_enum::NodeType}, storage::Storage};



#[derive(Savefile)]
pub struct ColorNode {
    x: f32,
    y: f32,
    id: String,
    color: [f32;4],
    input: bool,
    size: (u32,u32),
}

impl Default for ColorNode {
    fn default() -> Self {
        ColorNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            color: [1.0;4],
            input: false,
            size: (1,1)
        }
    }
}
impl MyNode for ColorNode {
    fn path(&self) -> Vec<&str> {
        vec!["IO"]
    }

    
    fn set_id(&mut self, id: String) {
        self.id = id;
    }


    fn savefile_version() -> u32 {0}

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
        NodeType::SolidColor
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            ColorNode::savefile_version(),
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
    

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn run(&mut self, storage: &mut Storage, map: HashMap::<String, String>, renderer: &mut Renderer) -> bool {
        
        let output_id = self.output_id(self.outputs()[0].clone());
        

        let fragment_shader_src = 
            r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;
            uniform vec4 c;

            void main() {
            color = c;
            }
            "#;

    
    if self.input && self.inputs().len() > 0 {
        let input_id = self.input_id(self.inputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };
    self.size = match storage.get_texture(get_output) {
        Some(a) => {(a.width(), a.height())},
        None => {return false},
    };
    }
    

    storage.gen_frag_shader(fragment_shader_src.to_string()).unwrap();
    storage.create_and_set_texture(self.size.0, self.size.1, output_id.clone());
    
    
    let texture: &glium::Texture2d = match storage.get_texture(&output_id) {
        Some(a) => a,
        None => {return false;},
    };

    let shader = storage.get_frag_shader(fragment_shader_src.to_string()).unwrap();

            let uniforms = uniform! {
                tex: texture,
                c: self.color,
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
        ui.text_wrapped("basic node, for debugging purposes")
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer) {
        ui.columns(2, "color col", true);
        ui.checkbox("use input texture for dimensions", &mut self.input);
        ui.disabled(self.input, || {
            let mut input_val = [self.size.0 as i32,self.size.1 as i32];
            ui.input_int2("dimensions (w,h)", &mut input_val).build();
            self.size = (input_val[0].max(1) as u32, input_val[1].max(1) as u32);
        });
        ui.next_column();
        ui.set_next_item_width(0.7*ui.content_region_avail()[0].min(ui.content_region_avail()[1]));
        ui.color_picker4_config("color", &mut self.color)
        .mode(imgui::ColorPickerMode::HueBar)
        .display_hex(true)
        .display_hsv(true)
        .display_rgb(true)
        .build()
        ;
        ui.set_window_font_scale(1.0);
    }

}
