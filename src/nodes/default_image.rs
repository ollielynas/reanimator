use std::{any::Any, collections::HashMap, hash::Hash, path::PathBuf};
use image::EncodableLayout;
use imgui_glium_renderer::Renderer;
use crate::{
    node::{random_id,MyNode},
    storage::Storage,
};
use glium::{texture::RawImage2d, Texture2d};
use image;
use imgui::text_filter;
use savefile::{save_file, SavefileError};

use super::node_enum::NodeType;


#[derive(Savefile)]
pub struct DefaultImage {
    x: f32,
    y: f32,
    id: String,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    texture_cache: Option<u64>,
}

impl Default for DefaultImage {
    fn default() -> Self {
        DefaultImage {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            texture_cache: None,
        }
    }
}

impl MyNode for DefaultImage {

    fn savefile_version() -> u32 {
        0
    }

    fn path(&self) -> Vec<&str> {
        vec!["load"]
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::DefaultImageOut
    }



    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id()+".bin"),
            DefaultImage::savefile_version(),
            self,
        );
    }


    fn inputs(&self) -> Vec<String> {
        vec![]
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Default Image".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn run(&mut self, storage: &mut Storage, map: HashMap<String, String>, renderer: &mut Renderer) -> bool {
        let output_id = self.output_id(self.outputs()[0].clone());


        if self.texture_cache.is_none() || !storage.cached_texture_exists(self.texture_cache.unwrap()) {
        let image = image::load_from_memory_with_format(
            include_bytes!("generic-image-placeholder.png"),
            image::ImageFormat::Png,
        )
        .unwrap().flipv().into_rgba8();


        let not_texture = RawImage2d::from_raw_rgba(image.as_bytes().to_vec(), (image.width(), image.height()));
        // let a: HashMap<Texture2d, String> = HashMap::new();
        let texture: Texture2d = Texture2d::new(&storage.display, not_texture).unwrap();
        self.texture_cache = Some( storage.cache_texture(texture));
    }
        
        storage.set_id_of_cached_texture(self.texture_cache.unwrap(), output_id);
            // storage.set_texture(output_id,  texture);
        

        return true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}
