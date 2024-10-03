use crate::{
    node::{random_id, MyNode},
    storage::Storage,
    widgets::link_widget,
};
use glium::{texture::RawImage2d, Texture2d};
use image;
use image::EncodableLayout;
use imgui::text_filter;
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;

use std::{any::Any, collections::HashMap, hash::Hash, path::PathBuf};

use crate::nodes::node_enum::NodeType;

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

    fn path(&self) -> Vec<&str> {
        vec!["IO", "Load"]
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
     

    fn type_(&self) -> NodeType {
        NodeType::DefaultImageOut
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
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

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let output_id =self.output_id(&self.outputs()[0]);;

        if self.texture_cache.is_none()
            || !storage.cached_texture_exists(self.texture_cache.unwrap())
        {
            let image = image::load_from_memory_with_format(
                &fastrand::choice([
                    include_bytes!("img/image-1.jpg").to_vec(),
                    include_bytes!("img/image-2.jpg").to_vec(),
                    include_bytes!("img/image-3.jpg").to_vec(),
                    include_bytes!("img/image-4.jpg").to_vec(),
                    include_bytes!("img/image-5.jpg").to_vec(),
                    include_bytes!("img/image-6.jpg").to_vec(),
                    include_bytes!("img/image-7.jpg").to_vec(),
                    include_bytes!("img/image-8.jpg").to_vec(),
                    include_bytes!("img/image-9.jpg").to_vec(),
                    include_bytes!("img/image-10.jpg").to_vec(),
                    include_bytes!("img/image-11.jpg").to_vec(),
                    include_bytes!("img/image-12.jpg").to_vec(),
                    include_bytes!("img/image-13.jpg").to_vec(),
                ])
                .unwrap(),
                image::ImageFormat::Jpeg,
            )
            .unwrap()
            .flipv()
            .into_rgba8();

            let not_texture = RawImage2d::from_raw_rgba(
                image.as_bytes().to_vec(),
                (image.width(), image.height()),
            );
            // let a: HashMap<Texture2d, String> = HashMap::new();
            let texture: Texture2d = Texture2d::new(&storage.display, not_texture).unwrap();
            self.texture_cache = Some(storage.cache_texture(texture));
        }

        storage.set_id_of_cached_texture(self.texture_cache.unwrap(), output_id);
        // storage.set_texture(output_id,  texture);

        return Ok(());
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        if ui.button("new image") {
            self.texture_cache = None;
        }
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("A random royalty free test image.");
        link_widget(ui, "source".to_owned(), "https://unsample.net/".to_owned());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
