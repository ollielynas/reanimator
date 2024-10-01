use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};
use glium::{index, texture::RawImage2d, Texture2d};
use anyhow::anyhow;
use image::EncodableLayout;
use image::{
    self,
    gif::{GifDecoder, GifReader},
    AnimationDecoder, DynamicImage, GenericImageView, ImageDecoder, ImageFormat, Rgba,
};
use imgui::text_filter;
use imgui_glium_renderer::Renderer;
use rfd::FileDialog;
use savefile::{load_file, save_file, SavefileError};
use std::{any::Any, collections::HashMap, fs, hash::Hash, io::Read, path::PathBuf};

use crate::nodes::node_enum::NodeType;

use super::apply_path_root;

const VERSION: u32 = 0;

#[derive(Savefile)]
pub struct LoadGifNode {
    x: f32,
    y: f32,
    id: String,
    pub path: Option<PathBuf>,
    length: f32,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    texture_cache: Vec<u64>,
}

impl Default for LoadGifNode {
    fn default() -> Self {
        LoadGifNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            texture_cache: vec![],
            path: None,
            length: 0.0,
        }
    }
}

impl MyNode for LoadGifNode {
    fn savefile_version() -> u32
    where
        Self: Sized,
    {
        0
    }

    fn path(&self) -> Vec<&str> {
        vec!["IO", "Load"]
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("load gif files");
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {
        ui.text(format!(
            "path: {}",
            match &self.path {
                Some(a) => {
                    a.as_path().to_str().unwrap()
                }
                None => "no path selected",
            }
        ));

        if ui.button("change path") {
            self.texture_cache = vec![];
            self.path = FileDialog::new().add_filter("", &["gif"]).pick_file();
            if let Some(ref mut path) = self.path {
                apply_path_root::set_root(path, &storage);
            }
        }

        ui.text(format!("length: {:.5}", self.length));
        ui.text(format!(
            "fps: {:.1}",
            (self.texture_cache.len() as f32) / self.length
        ));
    }

    fn type_(&self) -> NodeType {
        NodeType::LoadGif
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
        return vec!["Gif Output".to_string()];
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
        if self.path.is_none() {
            return Err(anyhow!("no path set"));
        }

        let output_id =self.output_id(&self.outputs()[0]);;

        // log::info!("{:?}", self.texture_cache);

        if let Some(path) = &self.path {
            if self.texture_cache.len() == 0 {
                self.length = 0.0;
                let file = match fs::File::open(apply_path_root::get_with_root(path, &storage)) {
                    Ok(a) => a,
                    Err(e) => {
                        return Err(anyhow!("file not found"));
                    }
                };

                let gif = match GifDecoder::new(file) {
                    Ok(a) => a,
                    Err(e) => {
                        return Err(anyhow!("unable to decode file to gif"));
                    }
                };

                let mut image = DynamicImage::new_rgba8(gif.dimensions().0, gif.dimensions().1);

                for a in gif.into_frames() {
                    if let Ok(frame) = a {
                        let (msu, msl) = frame.delay().numer_denom_ms();
                        self.length += (msu as f32 / msl as f32) / 1000.0;

                        // let image = frame.buffer();
                        let b = frame.into_buffer();
                        *image.as_mut_rgba8().unwrap() = b;

                        let not_texture = RawImage2d::from_raw_rgba(
                            image.flipv().as_bytes().to_vec(),
                            (image.dimensions().0, image.dimensions().1),
                        );

                        let texture: Texture2d =
                            Texture2d::new(&storage.display, not_texture).unwrap();
                        self.texture_cache.push(storage.cache_texture(texture));
                    }
                }
                // let a: HashMap<Texture2d, String> = HashMap::new();
            } else {
                // return false;
            }
        }

        if self.texture_cache.len() > 0 {
            let index = (self.texture_cache.len() as f64 * (storage.time % self.length as f64)
                / (self.length as f64))
                .floor() as usize;
            storage.set_id_of_cached_texture(self.texture_cache[index], output_id);
        }

        return Ok(());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
