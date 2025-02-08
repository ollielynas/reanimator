use std::any::Any;
use std::collections::HashMap;
use std::fs::{File};
use std::path::PathBuf;
use std::rc::Rc;

use crate::node::{random_id, MyNode};
use crate::nodes::node_enum::*;
use crate::render_nodes::RenderNodesParams;
use crate::storage;



use glium::{
    texture::{RawImage2d},
    uniforms::{MagnifySamplerFilter, SamplerBehavior},
    Texture2d,
};
use image::codecs::gif::GifEncoder;
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;
use glium::{BlitTarget, Rect, Surface};
use image::{Delay, DynamicImage, ImageBuffer, Rgba};

use imgui::{TextureId, Ui};
use imgui_glium_renderer::{Renderer, Texture};
use platform_dirs::UserDirs;
use rfd::FileDialog;
use savefile::prelude::*;
use storage::Storage;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
fn default_image() -> Vec<u8> {
    return vec![];
    // Image::from_file_with_format(include_bytes!("./generic-image-placeholder.png"), Some(ImageFormat::Png))
}

#[derive(EnumIter, Savefile, PartialEq, Clone)]
pub enum OutputType {
    LiveDisplay {
        run: bool,
        fps: i32,
        #[savefile_ignore]
        #[savefile_versions = "..0"]
        #[savefile_default_val = "0.0"]
        last_frame: f64,
    },

    RenderImage,
    RenderGif {
        record: bool,
        frames: Vec<Vec<u8>>,
        fps: f32,
        start_time: f32,
        length: f32,
    },
}

impl OutputType {
    fn name(&self) -> String {
        match self {
            OutputType::LiveDisplay {
                run: _,
                fps: _,
                last_frame: _,
            } => "Live Output",
            OutputType::RenderImage => "Render Image",
            OutputType::RenderGif {
                record: _,
                frames: _,
                fps: _,
                start_time: _,
                length: _,
            } => "Render Gif",
        }
        .to_string()
    }
}

#[derive(Savefile)]
pub struct OutputNode {
    x: f32,
    y: f32,
    id: String,
    output: OutputType,
    pub run_with_time: Vec<f64>,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub texture_id: Option<TextureId>,
}

impl Default for OutputNode {
    fn default() -> Self {
        OutputNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            output: OutputType::LiveDisplay {
                run: false,
                fps: 32,
                last_frame: 0.0,
            },
            run_with_time: vec![],
            texture_id: None,
        }
    }
}

impl MyNode for OutputNode {
    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let input_id = self.input_id(&self.inputs()[0]);
        self.run_with_time = vec![];
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        if self.texture_id.is_none() {
            self.texture_id = Some(renderer.textures().insert(Texture {
                texture: Rc::new(Texture2d::empty(&storage.display, 10, 10).unwrap()),
                sampler: SamplerBehavior {
                    // minify_filter: MinifySamplerFilter:,
                    magnify_filter: MagnifySamplerFilter::Nearest,
                    ..Default::default()
                },
            }));
        }

        if let Some(frame) = storage.get_texture(get_output) {
            if let Some(texture_id) = self.texture_id {
                if let Some(texture) = renderer.textures().get_mut(texture_id) {
                    if texture.texture.dimensions() != frame.dimensions() {
                        texture.texture = Rc::new(
                            Texture2d::empty(&storage.display, frame.width(), frame.height())
                                .unwrap(),
                        );
                    }

                    // let simple_frame_buffer = SimpleFrameBuffer::new(&storage.display, ColorA);
                    frame.as_surface().blit_color(
                        &Rect {
                            left: 0,
                            bottom: 0,
                            width: frame.width(),
                            height: frame.height(),
                        },
                        &texture.texture.as_surface(),
                        &BlitTarget {
                            left: 0,
                            bottom: texture.texture.height(),
                            width: texture.texture.width() as i32,
                            height: -(texture.texture.height() as i32),
                        },
                        MagnifySamplerFilter::Nearest,
                    );
                }
            }
        } else {
            return Err(anyhow!("failed to create texture"));
        }
        return Ok(());
    }

    fn as_any(&self) -> &dyn Any {
        self
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

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            OutputNode::savefile_version(),
            self,
        );
    }

    fn edit_menu_render(&mut self, ui: &Ui, renderer: &mut Renderer, _storage: &Storage) {
        let items = OutputType::iter().collect::<Vec<_>>();
        ui.columns(3, "3 col", true);
        // ui.set_column_width(0, ui.window_size()[0] * 0.2);

        if let Some(_cb) = ui.begin_combo("##", self.output.name()) {
            for cur in &items {
                if &self.output == cur {
                    // Auto-scroll to selected item
                    ui.set_item_default_focus();
                }
                // Create a "selectable"
                let clicked = ui
                    .selectable_config(cur.name())
                    .selected(&self.output == cur)
                    .build();
                // When item is clicked, store it
                if clicked {
                    self.output = cur.clone();
                }
            }
        }
        ui.next_column();
        match self.output {
            OutputType::LiveDisplay {
                ref mut run,
                ref mut fps,
                ref mut last_frame,
            } => {
                if ui.button("render single frame") {
                    self.run_with_time.push(ui.time());
                };
                // ui.io().add_input_character('\u{FFFF}');
                // ui.same_line();
                ui.checkbox("live render", run);
                ui.slider("fps", 0, 200, fps);
                if *run && ui.time() - *last_frame >= 1.0 / *fps as f64 {
                    *last_frame = ui.time();
                    self.run_with_time.push(ui.time());
                }
            }
            OutputType::RenderImage => {
                if ui.button("render") {
                    self.run_with_time.push(ui.time());
                };
                if let Some(image_id) = &self.texture_id {
                    if ui.button("download") {
                        let user_dirs = UserDirs::new();

                        let frame = &renderer.textures().get(*image_id).unwrap().texture;

                        if let Some(path) = FileDialog::new()
                            .set_can_create_directories(true)
                            .set_title("Save Image")
                            .set_directory(user_dirs.unwrap().download_dir)
                            .set_file_name("out.png")
                            .add_filter("image", &[".png"])
                            .save_file()
                        {
                            let img: RawImage2d<_> = frame.read();
                            let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
                                frame.width(),
                                frame.height(),
                                img.data.into_owned(),
                            )
                            .unwrap();
                            let img = DynamicImage::ImageRgba8(img);

                            let a = img.save(path);
                            log::info!("{:?}", a);
                        }
                    }
                }
            }
            OutputType::RenderGif {
                ref mut record,
                ref mut frames,
                ref mut fps,
                ref mut start_time,
                ref mut length,
            } => {
                ui.disabled(*record, || {
                    if ui.button("record gif") {
                        *record = true;
                        *frames = vec![];
                        self.run_with_time.push(*start_time as f64);
                    }
                    ui.input_float("fps", fps).build();
                    ui.input_float("length", length).build();
                    ui.input_float("start time", start_time).build();
                });

                if *record {
                    if let Some(image_id) = self.texture_id {
                        if frames.len() as f32 * 1.0 / *fps > *length {
                            *record = false;
                            // let

                            let user_dirs = UserDirs::new();

                            // let b2 = buffer.clone();

                            if let Some(path) = FileDialog::new()
                                .set_can_create_directories(true)
                                .set_title("Save Image")
                                .set_directory(user_dirs.unwrap().download_dir)
                                .set_file_name("out.gif")
                                .add_filter("", &[".gif"])
                                .save_file()
                            {
                                // fs::write(path.clone(), &[]);
                                // let buffer = match fs::read(path) {
                                //     Ok(p) => p,
                                //     Err(e) => {
                                //         log::error!("{e}");
                                //         return;
                                //     },
                                // };
                                // let mut buffer: Vec<u8> = vec![];

                                let buffer = File::create(path).unwrap();

                                let mut gif_encoder = GifEncoder::new_with_speed(
                                    buffer, // ((1.0/ *fps)*100.0) as i32
                                    30,
                                );
                                let image_dimensions = renderer
                                    .textures()
                                    .get(image_id)
                                    .unwrap()
                                    .texture
                                    .dimensions();
                                for f in &*frames {
                                    let buffer = ImageBuffer::from_vec(
                                        image_dimensions.0,
                                        image_dimensions.1,
                                        f.to_vec(),
                                    );
                                    let frame = image::Frame::from_parts(
                                        buffer.unwrap(),
                                        0,
                                        0,
                                        Delay::from_numer_denom_ms(1000, *fps as u32),
                                    );

                                    gif_encoder.encode_frame(frame);
                                    // log::info!("a");
                                }
                                gif_encoder
                                    .set_repeat(image::codecs::gif::Repeat::Infinite)
                                    .unwrap();

                                // log::info!("{:?}", buffer);
                            }
                        } else {
                            self.run_with_time
                                .push((*start_time + ((*frames).len() + 1) as f32 / *fps) as f64);
                            let texture = &renderer.textures().get(image_id).unwrap().texture;

                            let img: RawImage2d<_> = texture.read();
                            let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
                                texture.width(),
                                texture.height(),
                                img.data.into_owned(),
                            )
                            .unwrap();
                            let img = DynamicImage::ImageRgba8(img);

                            let data = img.into_bytes();
                            frames.push(data);
                            // let data2 =
                        }
                    }
                }
            }
        }
        ui.next_column();
        if let Some(image_id) = self.texture_id {
            let image_dimensions_bad = renderer
                .textures()
                .get(image_id)
                .unwrap()
                .texture
                .dimensions();
            ui.text(format!("image size: {image_dimensions_bad:?}"));
            // let pos = ui.cursor_pos();
            let avail = ui.content_region_avail();
            let image_dimensions = [image_dimensions_bad.0 as f32, image_dimensions_bad.1 as f32];

            let scale = (avail[0] / image_dimensions[0]).min(avail[1] / image_dimensions[1]) * 0.95;
            if scale != 0.0 && image_dimensions[0] != 0.0 && image_dimensions[1] != 0.0 {
                ui.invisible_button(
                    "custom_button",
                    [image_dimensions[0] * scale, image_dimensions[1] * scale],
                );
                let draw_list = ui.get_window_draw_list();

                draw_list
                    .add_image(image_id, ui.item_rect_min(), ui.item_rect_max())
                    .build();
                // ui.get_window_draw_list().add_image(image_id,
                //     [pos[0], pos[1]], [pos[0]+180.0, pos[1] + 180.0]).build();
                // ui.image_button("image", image_id, [image_dimensions[0] * scale, image_dimensions[1] * scale]);
            }
        }
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["Frame Input".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec![];
    }

    fn savefile_version() -> u32 {
        1
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn render_in_node(&self, ui: &Ui, ui_scale: f32,  renderer: &mut Renderer, params: &mut RenderNodesParams) {
            params
                .time_list
                .append(&mut self.run_with_time.clone());
        
    if let Some(image_id) = self.texture_id {
        let avail = [50.0 * ui_scale, 50.0 * ui_scale];
        let image_dimensions_bad = renderer
            .textures()
            .get(image_id)
            .unwrap()
            .texture
            .dimensions();
        let image_dimensions =
            [image_dimensions_bad.0 as f32, image_dimensions_bad.1 as f32];

        let scale = (avail[0] / image_dimensions[0])
            .min(avail[1] / image_dimensions[1]);
        if scale != 0.0
            && image_dimensions[0] != 0.0
            && image_dimensions[1] != 0.0
        {
            if ui.image_button(
                "image",
                image_id,
                [image_dimensions[0] * scale, image_dimensions[1] * scale],
            ) {
                // params.time_list.push(ui.time());
            }
        }
    }
    }
}
