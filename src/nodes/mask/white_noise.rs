use std::{any::Any, collections::HashMap, ops::RangeBounds, path::PathBuf};

use glium::{uniform, DrawParameters, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, nodes::{node_enum::NodeType}, storage::Storage};



#[derive(Savefile)]
pub struct WhiteNoiseNode {
    x: f32,
    y: f32,
    id: String,
    input: bool,
    size: (u32,u32),
    #[savefile_versions="1.."]
    #[savefile_default_val="0"]
    seed: i32,
}

impl Default for WhiteNoiseNode {
    fn default() -> Self {
        WhiteNoiseNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            input: false,
            size: (1,1),
            seed: 0,
        }
    }
}
impl MyNode for WhiteNoiseNode {
    fn path(&self) -> Vec<&str> {
        vec!["Mask"]
    }

    fn savefile_version() -> u32 {1}

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
        NodeType::WhiteNoise
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            WhiteNoiseNode::savefile_version(),
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



    fn run(&mut self, storage: &mut Storage, map: HashMap::<String, String>, renderer: &mut Renderer) -> bool {
        
        let output_id = self.output_id(self.outputs()[0].clone());
        

        let fragment_shader_src = 
            r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;
            
            uniform sampler2D tex;
            uniform float time;
            
            float random (vec2 st) {
            return fract(sin(dot(st.xy,
                                vec2(12.9898,78.233)))*
                43758.5453123);
            }

            float PHI = 1.61803398874989484820459;  // Φ = Golden Ratio   

            float gold_noise(vec2 xy, float seed){
                return fract(tan(distance(xy*PHI, xy)*seed)*xy.x);
            }


            void main() {

            /* float rnd = gold_noise( v_tex_coords, time ); */
            float rnd = random( v_tex_coords * time );

            color = vec4(rnd);
            }
            "#;

    
    if self.input && self.inputs().len() > 0 {
        let input_id = self.input_id(self.inputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };
    self.size = match storage.get_texture(get_output) {
        Some(a) => {(a.height(), a.width())},
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
                time: storage.time as f32,
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
        ui.text_wrapped("source of white noise");
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer) {
        ui.checkbox("use input texture for dimensions", &mut self.input);
        ui.disabled(self.input, || {
            let mut input_val = [self.size.0 as i32,self.size.1 as i32];
            ui.input_int2("dimensions (w,h)", &mut input_val).build();
            self.size = (input_val[0].max(1) as u32, input_val[1].max(1) as u32);
        });
        ui.input_int("seed", &mut self.seed).build();
        if ui.button("gen random seed") {
            self.seed = fastrand::i32(i32::MIN..i32::MAX);
        }
    }

}
