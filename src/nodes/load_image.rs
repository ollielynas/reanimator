use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};
use glium::{texture::RawImage2d, Texture2d};
use image::{self, ImageFormat};
use image::EncodableLayout;
use imgui::text_filter;
use imgui_glium_renderer::Renderer;
use rfd::FileDialog;
use savefile::{load_file, save_file, SavefileError};
use std::{any::Any, collections::HashMap, fs, hash::Hash, path::PathBuf};

use super::node_enum::NodeType;

const VERSION: u32 = 0;

#[derive(Savefile)]
pub struct LoadImage {
    x: f32,
    y: f32,
    id: String,
    path: Option<PathBuf>,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    texture_cache: Option<u64>,
}

impl Default for LoadImage {
    fn default() -> Self {
        LoadImage {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            texture_cache: None,
            path: None,
        }
    }
}

impl MyNode for LoadImage {
    fn savefile_version() -> u32
    where
        Self: Sized,
    {
        0
    }

    fn path(&self) -> Vec<&str> {
        vec!["IO", "Image"]
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("this node allows you to load static images");
        ui.text_wrapped("the following image types are supported:");
        ui.bullet_text("Png ");
        ui.bullet_text("Jpeg");
        ui.bullet_text("WebP");
        ui.bullet_text("Tiff");
        ui.bullet_text("Tga ");
        ui.bullet_text("Bmp ");
        ui.bullet_text("Ico ");
        ui.bullet_text("Hdr ");
        ui.bullet_text("Pnm ");
        ui.bullet_text("Farbfeld");
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer) {
        ui.text(format!("path: {}", match &self.path {Some(a) => {
            a.as_path().to_str().unwrap()
        }
        None => "no path selected"
    }));

    if ui.button("change path") {
        self.texture_cache = None;
        self.path = FileDialog::new().add_filter("", &[
            "png",
            "jpg",
            "jepg",
            "webp",
            "tiff",
            "tif",
            "tga",
            "bmp",
            "ico",
            "hdr",
            "pbm", "pam", "ppm", "pgm",
            "ff"
            ]).pick_file();
    }

    }

    fn type_(&self) -> NodeType {
        NodeType::LoadImageType
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            VERSION,
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        vec![]
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Selected Image".to_string()];
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
        if self.path.is_none() {
            return false;
        }

        let output_id = self.output_id(self.outputs()[0].clone());

        // println!("{:?}", self.texture_cache);

        if let Some(path) = &self.path {
            if self.texture_cache.is_none()
                || !storage.cached_texture_exists(self.texture_cache.unwrap())
            {
                let bytes = match fs::read(path) {
                    Ok(a) => a,
                    Err(e) => {
                        println!("{e}");
                        return false;
                    }
                };
                let image = match image::load_from_memory(&bytes) {
                    Ok(a) => a,
                    Err(e) => {
                        println!("{e}");
                        return false;
                    }
                }
                .flipv()
                .into_rgba8();
                let not_texture = RawImage2d::from_raw_rgba(
                    image.as_bytes().to_vec(),
                    (image.width(), image.height()),
                );
                // let a: HashMap<Texture2d, String> = HashMap::new();
                let texture: Texture2d = Texture2d::new(&storage.display, not_texture).unwrap();
                self.texture_cache = Some(storage.cache_texture(texture));
            } else {
                // return false;
            }
        }
        storage.set_id_of_cached_texture(self.texture_cache.unwrap(), output_id);
        // storage.set_texture(output_id,  texture);

        return true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
