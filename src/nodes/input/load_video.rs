use std::{any::Any, collections::HashMap, path::PathBuf};

use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
use glium::{texture::RawImage2d, uniform, DrawParameters, Rect, Surface};
use imgui_glium_renderer::Renderer;
use rfd::FileDialog;
use savefile::{save_file, SavefileError};

use crate::{
    node::{random_id, MyNode},
    storage::Storage,
    widgets::link_widget,
};

use crate::nodes::node_enum::NodeType;

use super::apply_path_root;

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

    fn savefile_version() -> u32 {
        0
    }

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
        NodeType::LoadVideo
    }

    fn id(&self) -> String {
        self.id.clone()
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
        ui.text(format!("frames: {}", self.frames.len(),));
        ui.text(format!("length: {}", self.length,));
        
        if self.paused {
            if ui.button("unpause") {
                self.paused = false;
            }
        }else {
            if ui.button("pause") {
                self.paused = true;
            }
        }

        ui.checkbox("loop", &mut self.do_loop);

        ui.text(format!("{}/{}", self.length * self.play_head as f32, self.length));
        ui.slider("Video %", 0.0, 1.0, &mut self.play_head);

        if ui.button("change path") {
            self.frames = vec![];
            self.path = FileDialog::new().pick_file();
            if let Some(ref mut path) = self.path {
                apply_path_root::set_root(path, &storage);
            }
        }
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


        if let Some(path) = &self.path {
            if self.frames.len() == 0 {
                match load_video_bytes(&apply_path_root::get_with_root(path, &storage)) {
                    Ok((length, width, height, frame_data)) => {
                        self.frames = frame_data;
                        self.length = length;
                        self.width = width;
                        self.height = height;
                    }
                    Err(e) => {
                        println!("{e}");
                    }
                };
            } else {
                // return false;
            }
        }

        storage.create_and_set_texture(self.width, self.height, output_id.clone());


        if self.last_time > storage.time  {
            self.last_time = 0.0;
            if  !self.autoplay {
            self.paused = false;
        }
        }

        if !self.paused {
            self.play_head += (storage.time - self.last_time) / self.length as f64;
            if self.play_head >= 1.0 {
                if self.do_loop {
                    self.play_head = 0.0;
                }else {
                self.play_head = 1.0;
                self.paused;
                }
            }
        }

        self.last_time = storage.time;

        if self.frames.len() == 0 {
            return false;
        }
        let data = &self.frames[(self.play_head * self.frames.len() as f64).floor().clamp(0.0, (self.frames.len() -1) as f64) as usize];
        if let Some(texture) = storage.get_texture(&output_id) {
            texture.write(Rect {
                left: 0,
                bottom:0,
                width: self.width,
                height: self.height,
            }, RawImage2d::from_raw_rgba_reversed(data, (self.width, self.height)));
        }

        return true;
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

pub fn load_video_bytes(path: &PathBuf) -> anyhow::Result<(f32, u32, u32, Vec<Vec<u8>>)> {
    let mut height = 0;
    let mut width = 0;
    let mut inital_timestamp = -10.0;
    let mut final_timestamp = 0.0;
    println!("loading video");
    let mut bytes = Vec::new();
    FfmpegCommand::new() // <- Builder API like `std::process::Command`
        .input(path.display().to_string()) // <- Convenient argument presets
        .create_no_window()
        .format("rawvideo")
        .output("pipe:1")
        .spawn()? // <- Uses an ordinary `std::process::Child`
        .iter()? // <- Iterator over all log messages and video output
        .for_each(|event: FfmpegEvent| {
            match event {
                FfmpegEvent::OutputFrame(frame) => {
                    println!("frame: {}x{}", frame.width, frame.height);
                    width = frame.width;
                    height = frame.height;
                    if inital_timestamp == -10.0 {
                        inital_timestamp = frame.timestamp;
                    };
                    if final_timestamp < frame.timestamp {
                        final_timestamp = frame.timestamp;
                    };
                    let pixels: Vec<u8> = frame.data; // <- raw RGB pixels! 🎨
                    bytes.push(pixels);
                }
                FfmpegEvent::Progress(progress) => {
                    println!("Current speed: {}x", progress.speed); // <- parsed progress updates
                }
                FfmpegEvent::Log(_level, msg) => {
                    println!("[ffmpeg] {}", msg); // <- granular log message from stderr
                }
                _ => {}
            }
        });

    Ok((final_timestamp - inital_timestamp, width, height, bytes))
}
