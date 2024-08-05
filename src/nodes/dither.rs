use std::{any::Any, collections::HashMap, path::PathBuf};

use blake2::digest::typenum::Pow;
use glium::{pixel_buffer::PixelBuffer, texture::{RawImage2d, Texture2dDataSource}, uniform, DrawParameters, Rect, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{node::{random_id, MyNode}, storage::Storage};

use super::node_enum::NodeType;

#[derive(Savefile, Debug)]

struct DitherPatternPos {
    x: i8,
    y: i8,
    multiplier: u8,
}


const PATTERN: [(&str, &str, u8); 8] = [
(
"Floyd-Steinberg",
r#"0 x 7
3 5 1
"#,
16_u8
),
(
"Jarvis, Judice, and Ninke",
r#"0 0 X 7 5 
3 5 7 5 3
1 3 5 3 1
"#,
48_u8
),
(
"Stucki",
r#"0 0 X 8 4
2 4 8 4 2
1 2 4 2 1
"#,
42_u8
),
(
"Atkinson",
r#"0 X 1 1 
1 1 1
0 1
"#,
8_u8
),
(
"Burkes",
r#"0 0 X 8 4 
2 4 8 4 2
"#,
32_u8
),

(
"Two-Row Sierra",
r#"0 0 X 4 3
1 2 3 2 1
"#,
16_u8
),
(
"Sierra Lite",
r#"0 x 2
1 1
"#,
16_u8
),
(
"Sierra ",
r#"0 0 X 5 3
2 4 5 4 2
0 2 3 2
"#,
32_u8
),

];


#[derive(Savefile)]
pub struct LinearErrorDitherNode {
    x: f32,
    y: f32,
    id: String,
    notation: String,
    pattern: Vec<DitherPatternPos>,
    devisor: i32,
    // image_vec: 
}

impl Default for LinearErrorDitherNode {
    fn default() -> Self {
        let mut new = LinearErrorDitherNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            notation: PATTERN[7].1.to_owned(),
            pattern: vec![],
            devisor: PATTERN[7].2 as i32,
        };
        new.load_notation();
        return new;
    }
}

impl LinearErrorDitherNode {
    fn load_notation(&mut self) {
        let mut offset: i8 = 0;
        let mut hit_x = false;

        self.pattern = vec![];

        let binding = self.notation.replace("\n", " | ");
        let bit = binding.split(" ");

        let mut line = 0;
        let mut x = 0;

        for b in bit {
            // println!("{b},{}", b.contains("\n"));
            match b {
                "0" if !hit_x => {
                    offset -= 1;
                    x += 1;
                }
                "|" if hit_x => {
                    line += 1;
                    x = 0;
                }
                "X" | "x" if !hit_x => {
                    hit_x = true;
                    x += 1;
                }

                a if hit_x && a.parse::<u8>().is_ok() => {
                    self.pattern.push(DitherPatternPos {
                        x: x + offset,
                        y: line,
                        multiplier: a.parse::<u8>().unwrap(),
                    });
                    x += 1;
                }

                _ => {}
            }
        }

        // for a in &self.pattern {
        //     println!("{a:?}");
        // }

    }
}

impl MyNode for LinearErrorDitherNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image","Advanced Shader"]
    }

    
    fn set_id(&mut self, id: String) {
        self.id = id;
    }


    fn savefile_version() -> u32 {0}

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
        NodeType::LinearErrorDither
    }


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            LinearErrorDitherNode::savefile_version(),
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

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer) {
        let space = ui.content_region_avail();
        ui.columns(2, "dither col", true);
        ui.input_text_multiline("Pattern", &mut self.notation, [space[0], space[1]]).build();
        if ui.is_item_edited() {
            self.load_notation();
        }
        ui.next_column();
        ui.input_int("devisor", &mut self.devisor).build();
        self.devisor  = self.devisor.clamp(1, 255);
        // if ui.button("build") {
        //     self.load_notation();
        // }
        if ui.button("pick from preset") {
            ui.open_popup("dither preset")
        }
        ui.popup("dither preset", ||{
            for (name, notation, devisor) in PATTERN.iter().rev() {
                if ui.button(name) {
                    self.devisor = *devisor as i32;
                    self.notation = notation.to_string();
                    self.load_notation();
                }
            }
        });
    }


    fn run(&mut self, storage: &mut Storage, map: HashMap::<String, String>, renderer: &mut Renderer) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };


    let texture_size:(u32, u32) = match storage.get_texture(get_output) {
        Some(a) => {(a.width(), a.height())},
        None => {return false},
    };
    

    storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());
    
    let texture: &glium::Texture2d = match storage.get_texture(get_output) {
        Some(a) => {a},
        None => {return false},
    };

    let mut data = texture.read_to_pixel_buffer().as_slice().read().unwrap_or_default().iter().map(|(r,g,b,a)| {
        // (((*r.max(g).max(b) as u16 + *r.min(g).min(b) as u16)/2) as i16,*a)
        (((*r as u16 + *g as u16 + *b as u16)/3) as i16,*a)
    }).collect::<Vec<(i16, u8)>>();

    // let can_bit_shift = (self.devisor as u32).is_power_of_two();

    // let bit_shifts = (self.devisor as f32).powf(0.5) as i32;


    let mut error;

        let width = texture_size.0;
        for i in 0..data.len() {

            let x = i as i32 % width as i32;
            let black = data[i].0 < 127;
            
            error = data[i].0 as i32 - if black {0} else {255};
            // println!("{error}");
            // if error == 0 {
            //     data[i].0 = if black {0} else {255};
            //     continue;
            // }
            for p in &self.pattern {
                // i16::MAX
                if x >= p.x as i32 && x + (p.x as i32) < width as i32 {
                    let index = i as i32 + p.x as i32 + p.y as i32 * width as i32;
                    if index >= 0 && index < data.len() as i32 {
                        data[index as usize].0 = (data[index as usize].0 as i32 + ((( error) * p.multiplier as i32) / self.devisor)).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                    }
                }
            }
            data[i].0 = if black {0} else {255};
            // error = 0;
        }

        
        let texture2 = storage.get_texture(&output_id).unwrap();
        let image2d = RawImage2d::from_raw_rgba(data.iter().flat_map(|(b,a)| [*b as f32 / 255.0,*b as f32 / 255.0,*b as f32 / 255.0,*a as f32 / 255.0]).collect(), texture.dimensions());


            // println!("{:?} {}", texture2.dimensions(), texture2.width());
            texture2.write(Rect {
                left: 0,
                bottom: 0,
                width: texture2.width(),
                height: texture2.height(),
            }, image2d);

        return true;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Applies dither effect")
    }
}



/*
oooooooooo.                                                oooooooooo.    o8o      .   oooo                           
`888'   `Y8b                                               `888'   `Y8b   `"'    .o8   `888                           
 888     888  .oooo.   oooo    ooo  .ooooo.  oooo d8b       888      888 oooo  .o888oo  888 .oo.    .ooooo.  oooo d8b 
 888oooo888' `P  )88b   `88.  .8'  d88' `88b `888""8P       888      888 `888    888    888P"Y88b  d88' `88b `888""8P 
 888    `88b  .oP"888    `88..8'   888ooo888  888           888      888  888    888    888   888  888ooo888  888     
 888    .88P d8(  888     `888'    888    .o  888           888     d88'  888    888 .  888   888  888    .o  888     
o888b0od8P'  `Y888""8o     .8'     `Y8bod8P' d888b         o888bo0d8P'   o888o   "888" o888o o888o `Y8bod8P' d888b    
x                      .o..P'                                                                                         
x                      `Y8P'                                                                                          
*/

#[derive(Savefile)]
pub struct BayerDitherNode {
    x: f32,
    y: f32,
    id: String,
    size: i32,
}

impl Default for BayerDitherNode {
    fn default() -> Self {
        BayerDitherNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            size: 4,
        }
    }
}
impl MyNode for BayerDitherNode {
    fn path(&self) -> Vec<&str> {
        vec!["Image","Advanced Shader"]
    }

    
    fn set_id(&mut self, id: String) {
        self.id = id;
    }


    fn savefile_version() -> u32 {0}

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
        NodeType::BayerDither
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer) {
        ui.text(format!("size: {}x{}", self.size, self.size));
        ui.separator();
        if ui.button("2x2") {
            self.size = 2;
        }
        if ui.button("4x4") {
            self.size = 4;
        }
        if ui.button("8x8") {
            self.size = 8;
        }
    }




    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            LinearErrorDitherNode::savefile_version(),
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



    fn run(&mut self, storage: &mut Storage, map: HashMap::<String, String>, renderer: &mut Renderer) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };



        let fragment_shader_src = include_str!("dither.glsl");

    let texture_size:(u32, u32) = match storage.get_texture(get_output) {
        Some(a) => {(a.width(), a.height())},
        None => {return false},
    };
    

    storage.gen_frag_shader(fragment_shader_src.to_string()).unwrap();
    storage.create_and_set_texture(texture_size.0, texture_size.1, output_id.clone());
    
    let texture: &glium::Texture2d = match storage.get_texture(get_output) {
        Some(a) => {a},
        None => {return false},
    };

    let shader = storage.get_frag_shader(fragment_shader_src.to_string()).unwrap();

            let uniforms = uniform! {
                tex: texture,
                u_resolution: [texture_size.0 as f32, texture_size.1 as f32],
                size: self.size,
            };
            let texture2 = storage.get_texture(&output_id).unwrap();
            texture2.as_surface().draw(&storage.vertex_buffer, &storage.indices, shader, &uniforms,
                            &DrawParameters {
                                dithering: true,
                                ..Default::default()
                            }
                            ).unwrap();

        return true;
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Applies dither effect")
    }
}
