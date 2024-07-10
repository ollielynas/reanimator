use std::{any::Any, collections::HashMap, path::PathBuf};

use glium::{uniform, uniforms::{self, Uniforms}, BlitTarget, DrawParameters, Rect, Surface};
use imgui::{sys::ImColor, ImColor32};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};

use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};

use super::node_enum::NodeType;

#[derive(Savefile)]
pub struct LayerNode {
    x: f32,
    y: f32,
    id: String,
    layers: Vec<[f32; 4]>,
    base_texture_size: (u32, u32),
}

impl Default for LayerNode {
    fn default() -> Self {
        LayerNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            layers: vec![],
            base_texture_size: (0, 0),
        }
    }
}
impl MyNode for LayerNode {
    fn path(&self) -> Vec<&str> {
        vec!["Combine"]
    }

    fn savefile_version() -> u32 {
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
        NodeType::Layer
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            LayerNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        let mut v = vec!["Base".to_string()];
        for i in 0..self.layers.len() {
            v.push(format!("Layer {}", i + 1));
        }
        return v;
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

        let output_id = self.output_id(self.outputs()[0].clone());
        
        let base_input_key = match map.get(&self.input_id(self.inputs()[0].clone())) {
            Some(a) => a,
            None => {
                return false;
            }
        };
        let mut inputs = vec![];
        for i in 1..self.inputs().len() {
            let input_id = self.input_id(self.inputs()[i].clone());
            let get_output = match map.get(&input_id) {
                Some(a) => a,
                None => {
                    return false;
                }
            };
            inputs.push(get_output);
        }

        // let layer_string = (0..self.layers.len())
        //     .map(|x|
        //         format!(
        //             "uniform sampler2D layer_{x};uniform vec2 layer_{x}_pos;uniform vec2 layer_{x}_size;uniform vec2 layer_{x}_target_size;"
        //         )
        //     )
        //     .collect::<String>();
        // let code = (0..self.layers.len())
        //     .map(|x| format!("color = add_layer(layer_{x}_pos, layer_{x}_size,layer_{x}_target_size,  color, layer_{x});"))
        //     .collect::<String>();

        // let fragment_shader_src =
        //     r#"

        //     #version 140

        //     in vec2 v_tex_coords;
        //     out vec4 color;
            
        //     uniform sampler2D base_texture;
            
        //     LAYERS

        //     vec4 add_layer(vec2 target_pos, vec2 size, vec2 target_size, vec4 base_color, sampler2D tex_layer) {
        //         if (v_tex_coords.x < target_pos.x || v_tex_coords.y < target_pos.y || v_tex_coords.x > target_pos.x + target_size.x || v_tex_coords.y > target_pos.y + target_size.y) {
        //             return base_color;
        //         }

        //         vec4 layer_color = texture(tex_layer, ((v_tex_coords - target_pos) * size / target_size));

        //         return layer_color;
        //     }

        //     void main() {
        //         color = texture(base_texture, v_tex_coords);
        //         CODE
        //     }
        //     "#
        //         .replace("LAYERS", &layer_string)
        //         .replace("CODE", &code);
    
        let fragment_shader_src2 =
            r#"

            #version 140

            in vec2 v_tex_coords;
            out vec4 color;
            
            uniform sampler2D base_texture;
            
            uniform sampler2D layer;
            uniform vec2 layer_pos;
            uniform vec2 base_size;
            uniform vec2 layer_size;
            uniform vec2 layer_target_size;

            vec4 add_layer(vec2 target_pos, vec2 size, vec2 target_size, vec4 base_color) {
                if (v_tex_coords.x * base_size.x < target_pos.x || v_tex_coords.y * base_size.y < target_pos.y || v_tex_coords.x * base_size.x > target_pos.x + target_size.x || v_tex_coords.y * base_size.y > target_pos.y + target_size.y) {
                    return base_color;
                }

                vec4 layer_color = texture(layer, ((v_tex_coords * base_size - target_pos) * layer_size / target_size)/layer_size);

                float p1 = layer_color.a;
                float p2 = base_color.a;
                vec3 c1 = layer_color.xyz;
                vec3 c2 = base_color.xyz;
                
                return vec4((p1*c1+p2*c2-p1*p2*c2)/(p1+p2-p1*p2),p1+p2-p1*p2);

            }
        
            void main() {
                
                color = texture(base_texture, v_tex_coords);

                color = add_layer(layer_pos, layer_size,layer_target_size,  color);
            }
            "#;

        // println!("{}", fragment_shader_src);

        let mut sizes: Vec<(u32, u32)> = vec![];

        for get_output in &inputs {
            let texture_size: (u32, u32) = match storage.get_texture(get_output) {
                Some(a) => (a.height(), a.width()),
                None => {
                    return false;
                }
            };
            sizes.push(texture_size);
        }

        storage
            .gen_frag_shader(fragment_shader_src2.to_string())
            .unwrap();

        storage.create_and_set_texture(
            self.base_texture_size.0,
            self.base_texture_size.1,
            output_id.clone(),
        );

        let shader = storage
            .get_frag_shader(fragment_shader_src2.to_string())
            .unwrap();

        let base_texture: &glium::Texture2d = match storage.get_texture(base_input_key) {
            Some(a) => a,
            None => {
                return false;
            }
        };
        
        
        self.base_texture_size = base_texture.dimensions();
        let texture2 = storage.get_texture(&output_id).unwrap();


        base_texture.as_surface().blit_color(&Rect {
            left: 0,
            bottom: 0,
            width: base_texture.width(),
            height: base_texture.height(),
        }, &texture2.as_surface(), &BlitTarget {
            left: 0,
            bottom: 0,
            width: texture2.width() as i32,
            height: texture2.height() as i32,
        }, uniforms::MagnifySamplerFilter::Linear);
        
        for x in 0..self.layers.len() {
            let layer = self.layers[x];
            let id = inputs[x];

            let texture: &glium::Texture2d = match storage.get_texture(&id) {
                Some(a) => a,
                None => {
                    return false;
                }
            };
            println!("{:?} {:?} {:?}", [layer[0], layer[1]],[layer[2], layer[3]],[texture.dimensions().0 as f32, texture.dimensions().1 as f32]);
            let uniforms = uniform! {
                base_texture: texture2,
                base_size: [self.base_texture_size.0 as f32, self.base_texture_size.1 as f32],
                layer_pos: [layer[0], layer[1]],
                layer_target_size: [layer[2], layer[3]],
                layer_size: [texture.dimensions().0 as f32, texture.dimensions().1 as f32],
                layer: texture,
            };

            texture2
            .as_surface()
            .draw(
                &storage.vertex_buffer,
                &storage.indices,
                shader,
                &uniforms,
                &(DrawParameters {
                    ..Default::default()
                }),
            )
            .unwrap();
        }
        
        return true;
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer) {
        ui.columns(2, "", true);
        ui.text(format!("base size: {:?}", self.base_texture_size));
        let mut focus = 999_usize;
        let mut hover = 999_usize;
        for (i, l) in self.layers.iter_mut().enumerate() {
            let mut xy = [l[0], l[1]];
            let mut wh = [l[2], l[3]];
            ui.group(|| {
            
                ui.input_float2(format!("target <x,y> position ({i})"), &mut xy).build();
                ui.input_float2(format!("target <width, height> ({i})"), &mut wh).build();
            });
            *l = [xy[0],xy[1],wh[0],wh[1]];
            if ui.is_item_focused() {
                focus = i;
            }
            if ui.is_item_hovered() {
                hover = i;
            }
        }

        if ui.button("add layer") {
            self.layers.push([
                (self.base_texture_size.0 as f32) * 0.25,
                (self.base_texture_size.1 as f32) * 0.25,
                (self.base_texture_size.0 as f32) * 0.25,
                (self.base_texture_size.0 as f32) * 0.25,
            ]);
        }

        ui.next_column();

        let draw_list = ui.get_window_draw_list();
        let pos = ui.cursor_screen_pos();
        let avail = ui.content_region_avail();
        let image_dimensions = [
            self.base_texture_size.0 as f32,
            self.base_texture_size.1 as f32,
        ];

        let scale = (avail[0] / image_dimensions[0]).min(avail[1] / image_dimensions[1]);

        draw_list
            .add_rect(
                pos,
                [
                    pos[0] + image_dimensions[0] * scale,
                    pos[1] + image_dimensions[1] * scale,
                ],
                ImColor32::from_rgba(80, 80, 80, 235),
            )
            .filled(true)
            .build();
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Allows textures to be layered on top of each other");
    }
}