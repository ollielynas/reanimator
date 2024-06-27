
use std::collections::HashMap;
use std::path::PathBuf;

use fastrand;
use image::{DynamicImage, ImageBuffer, Rgba};
use imgui::draw_list::Image;
use imgui::TextureId;
use imgui_glium_renderer::Texture;
use savefile::prelude::*;
use crate::nodes::node_enum::*;
use crate::node::{random_id, MyNode, VERSION};
use crate::storage;
use storage::Storage;
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};
fn default_image() -> Vec<u8> {
    return vec![];
    // Image::from_file_with_format(include_bytes!("./generic-image-placeholder.png"), Some(ImageFormat::Png))
}




#[derive(Savefile)]
pub struct OutputNode {
    x: f32,
    y: f32,
    id: String, 
}

impl Default for OutputNode {
    fn default() -> Self {
        OutputNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}

impl MyNode for OutputNode {

    fn run(&self, storage: &mut Storage, map: HashMap::<String, String>) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };
        if let Some(frame) = storage.get_texture(get_output) {
            let img: RawImage2d<_> =  frame.read();
            let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(frame.width(), frame.height(), img.data.into_owned()).unwrap();
            let img = DynamicImage::ImageRgba8(img).flipv();
    
            img.save("image.png").unwrap();
        }else {
            return false
        }
        return true;
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn path(&self) -> Vec<&str> {
        vec!["IO"]
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::Output
    }
    


    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            VERSION,
            self,
        );
    }
    
    fn edit_menu_render(&self) {
        
    }
    
    fn inputs(&self) -> Vec<String> {
        return vec!["Frame Input".to_string()];
        }
        
        fn outputs(&self) -> Vec<String> {
        return vec![];
    }
}