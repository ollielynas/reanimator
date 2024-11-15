use std::{any::Any, collections::HashMap, path::PathBuf};

use ffmpeg_sidecar::{
    command::FfmpegCommand,
};
use glium::{texture::RawImage2d, Rect};
use imgui_glium_renderer::Renderer;
use itertools::Itertools;
use rfd::FileDialog;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{
    node::{random_id, MyNode},
    storage::Storage,
    widgets::link_widget,
};

use crate::nodes::node_enum::NodeType;

use super::apply_path_root;
/// It would be good if this node transferred less data to and from the gpu
#[derive(Savefile)]
pub struct LoadVideoNode {
    x: f32,
    y: f32,
    id: String,
    path: Option<PathBuf>,
    #[savefile_ignore]
    frames: Vec<Vec<u8>>,
    playback_speed: f32,
    #[savefile_ignore]
    length: f32,
    #[savefile_ignore]
    paused: bool,
    #[savefile_ignore]
    play_head: f64,
    autoplay: bool,
    #[savefile_ignore]
    width: u32,
    #[savefile_ignore]
    height: u32,
    #[savefile_ignore]
    last_time: f64,
    do_loop: bool,
    #[savefile_versions = "1.."]
    custom_input: bool,
    #[savefile_versions = "1.."]
    custom_args: bool,

    #[savefile_versions = "1.."]
    custom_input_text: String,

    #[savefile_versions = "1.."]
    ffmpeg_args: String,
}

impl Default for LoadVideoNode {
    fn default() -> Self {
        LoadVideoNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            path: None,
            frames: vec![],
            playback_speed: 1.0,
            length: 0.0,
            paused: true,
            play_head: 0.0,
            autoplay: true,
            last_time: 0.0,
            width: 1,
            height: 1,
            do_loop: true,
            custom_input: false,
            custom_args: false,
            custom_input_text: String::new(),
            ffmpeg_args: String::new(),
        }
    }
}

impl MyNode for LoadVideoNode {
    fn path(&self) -> Vec<&str> {
        vec!["IO", "Load"]
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

    fn savefile_version() -> u32 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
     

    fn type_(&self) -> NodeType {
        NodeType::LoadVideo
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            LoadVideoNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec![];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Video Out".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, storage: &Storage) {
        let mut changed_path = false;
        ui.disabled(self.custom_input, || {
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
                self.frames = vec![];
                self.path = FileDialog::new().pick_file();
                if let Some(ref mut path) = self.path {
                    apply_path_root::set_root(path, &storage);
                    changed_path = true;
                }
            }
        });
        ui.checkbox("use custom input", &mut self.custom_input);
        ui.checkbox("use custom args", &mut self.custom_args);
        if self.custom_input {
            ui.text("-i ");
            ui.same_line();
            ui.input_text("custom input", &mut self.custom_input_text)
                .build();
        }
        if self.custom_args {
            let h = ui.calc_text_size_with_opts(
                "x".repeat(self.ffmpeg_args.split("\n").collect::<Vec<&str>>().len()),
                false,
                0.1,
            )[1] + ui.clone_style().frame_padding[1] * 2.0;
            ui.input_text_multiline(
                "args",
                &mut self.ffmpeg_args,
                [ui.content_region_avail()[0], h],
            )
            .build();
        }

        ui.text(format!("frames: {}", self.frames.len(),));
        ui.text(format!("length: {}", self.length,));

        if ui.button("load file") || changed_path {
            self.load_assets(storage);
        }

        if ui.is_item_hovered() {
            ui.tooltip_text("(this could take a sec)");
        }

        if self.paused {
            if ui.button("unpause") {
                self.paused = false;
            }
        } else {
            if ui.button("pause") {
                self.paused = true;
            }
        }

        ui.checkbox("loop", &mut self.do_loop);

        ui.text(format!(
            "{}/{}",
            self.length * self.play_head as f32,
            self.length
        ));
        // if !(0.0..=1.0).contains(&self.play_head) {
        //     self.play_head = 0.0;
        // }
        ui.slider("Video %", 0.0, 1.0, &mut self.play_head);
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        _map: HashMap<String, String>,
        _renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        if self.path.is_none() {
            return Err(anyhow!("path is none"));
        }

        let output_id =self.output_id(&self.outputs()[0]);;

        storage.create_and_set_texture(self.width, self.height, output_id.clone());

        if self.last_time > storage.time {
            self.last_time = 0.0;
            if !self.autoplay {
                self.paused = false;
            }
        }

        if !self.paused {
            self.play_head += (storage.time - self.last_time) / self.length as f64;
            if self.play_head >= 1.0 {
                if self.do_loop {
                    self.play_head = 0.0;
                } else {
                    self.play_head = 1.0;
                    self.paused;
                }
            }
        }

        self.last_time = storage.time;

        if self.frames.len() == 0 {
            return Err(anyhow!("no frames loaded"));
        }
        let data = &self.frames[(self.play_head * self.frames.len() as f64)
            .floor()
            .clamp(0.0, (self.frames.len() - 1) as f64) as usize];
        if data.len() as u32 == self.height * self.width * 4 {
            if let Some(texture) = storage.get_texture(&output_id) {
                texture.write(
                    Rect {
                        left: 0,
                        bottom: 0,
                        width: self.width,
                        height: self.height,
                    },
                    RawImage2d::from_raw_rgba_reversed(data, (self.width, self.height)),
                );
            }
        }

        return Ok(());
    }

    fn load_assets(&mut self, storage: &Storage) {
        self.frames = vec![];
            match load_video_bytes(
                &if self.custom_input {
                    PathBuf::new()
                } else {
                    apply_path_root::get_with_root(&self.path.clone().unwrap_or_default(), &storage)
                },
                if self.custom_input {
                    self.custom_input_text.clone()
                } else {
                    String::new()
                },
                if self.custom_args {
                    self.ffmpeg_args.clone()
                } else {
                    String::new()
                },
            ) {
                Ok((length, width, height, frame_data)) => {
                    self.frames = frame_data;
                    self.length = length;
                    self.width = width;
                    self.height = height;
                }
                Err(e) => {
                    log::error!("{e}");
                }
            };
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("loads a video file using ffmpef");
        ui.text("");
        link_widget(
            ui,
            "about ffmpeg".to_owned(),
            "https://ffmpeg.org/".to_owned(),
        );
        ui.text("");
        link_widget(
            ui,
            "ffmpeg-sidecar".to_owned(),
            "https://crates.io/crates/ffmpeg-sidecar".to_owned(),
        );
    }
}

/// ## returns: (length, width, height, frames)
pub fn load_video_bytes(
    path: &PathBuf,
    custom_input: String,
    custom_args: String,
) -> anyhow::Result<(f32, u32, u32, Vec<Vec<u8>>)> {
    let mut height = 0;
    let mut width = 0;
    let mut inital_timestamp = -10.0;
    let mut final_timestamp = 0.0;
    let _finished = false;
    log::info!("loading video");
    let mut bytes = Vec::new();
    let mut binding = FfmpegCommand::new();
    let _args = custom_args.replace("\n", " ");
    let command = binding // <- Builder API like `std::process::Command`
        .hide_banner()
        .input(if path == &PathBuf::new() {
            custom_input
        } else {
            format!("{}", path.display().to_string().replace("\\", "/"))
        }) // <- Convenient argument presets
        .rawvideo()
        .pix_fmt("rgba")
        .spawn();
    let frames = command.unwrap().iter().unwrap().filter_frames();

    let mut frame_index = 0;
    for frame in frames {
        width = frame.width;
        height = frame.height;
        if inital_timestamp == -10.0 {
            inital_timestamp = frame.timestamp;
        };
        if final_timestamp < frame.timestamp {
            final_timestamp = frame.timestamp;
        };
        let mut pixels: Vec<u8> = frame.data; // <- raw RGB pixels! 🎨
        if pixels.len() as u32 == width * height * 3 {
            pixels = pixels
                .chunks_exact(3)
                .flat_map(|x| [x[0], x[1], x[2], 255])
                .collect::<Vec<u8>>();
        }
        // println!("{} - {} {}", pixels.len(), width * height * 4, width * height * 3);
        bytes.push(pixels);
        frame_index += 1;
    }

    if inital_timestamp == -10.0 {
        return Err(anyhow::Error::msg("no frames found"));
    }

    println!(
        "{:?}",
        ((
            final_timestamp - inital_timestamp,
            width,
            height,
            bytes.len(),
        ))
    );

    Ok((final_timestamp - inital_timestamp, width, height, bytes))
}
