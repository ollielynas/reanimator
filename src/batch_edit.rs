use std::{
    ffi::OsStr, fs::{self, read_dir}, io::Write, path::{Path, PathBuf}
};

use crate::{
    nodes::input::load_video::load_video_bytes, project::Project, sidebar::SidebarParams, user_info::UserSettings, widgets::link_widget
};

use glium::texture::RawImage2d;
use image::EncodableLayout;
use imgui::{sys::ImVec2, Ui};
use imgui_glium_renderer::Renderer;
use itertools::Itertools;
use numfmt::{Formatter, Precision, Scales};
use rfd::FileDialog;

use anyhow::anyhow;

#[derive(Savefile, Clone)]
pub struct MyFile {
    pub path: PathBuf,
    pub size: u64,
}

fn try_load_as_image(path: &PathBuf) -> Option<RawImage2d<'static, u8>> {
    let bytes = match fs::read(path) {
        Ok(a) => a,
        Err(e) => {
            log::error!("{e}");
            return None;
        }
    };
    let image = match image::load_from_memory(&bytes) {
        Ok(a) => a,
        Err(e) => {
            log::error!("{e}");
            return None;
        }
    }
    .flipv()
    .into_rgba8();

    let raw_image =
        RawImage2d::from_raw_rgba(image.as_bytes().to_vec(), (image.width(), image.height()));

    log::info!("{:?}", raw_image.format);

    return Some(raw_image);
}

impl MyFile {
    pub fn get_raw(&self) -> Vec<RawImage2d<u8>> {
        let mut output = vec![];

        if let Some(data) = try_load_as_image(&self.path) {
            output.push(data);
            return output;
        } else {
            return output;
        }
    }

    pub fn new(path: PathBuf) -> Option<MyFile> {
        let meta = fs::metadata(&path);
        if let Ok(metadata) = meta {
            Some(MyFile {
                path,
                size: metadata.len(),
            })
        } else {
            None
        }
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .unwrap_or(&OsStr::new("Error"))
            .to_str()
            .unwrap()
            .split(".")
            .next()
            .unwrap()
            .to_string()
    }
    pub fn type_(&self) -> String {
        self.path
            .extension()
            .unwrap_or(&OsStr::new(""))
            .to_str()
            .unwrap()
            .to_string()
    }
}

#[derive(Savefile)]
pub struct RunBatch {
    pub files: Vec<MyFile>,
    pub save_path: PathBuf,
    #[savefile_ignore]
    pub index: usize,
    #[savefile_ignore]
    pub run: bool,
}

fn recurse_files(path: impl AsRef<Path>) -> std::io::Result<Vec<PathBuf>> {
    let mut buf = vec![];
    let entries = read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path())?;
            buf.append(&mut subdir);
        }

        if meta.is_file() {
            buf.push(entry.path());
        }
    }

    Ok(buf)
}

impl Project {

    pub fn process_file(&mut self, renderer: &mut Renderer, input_file: PathBuf, output_file: PathBuf) -> anyhow::Result<()> {

        
        
        let (length, width, height, frames) = load_video_bytes(&input_file, String::new(), String::new())?;
        let mut command = ffmpeg_sidecar::command::FfmpegCommand::new()
        .format("rawvideo")
        .size(width, height)
        .pix_fmt("rgba")
        .input("pipe:0")
        .output(output_file.display().to_string())
        .duration(length.to_string())
        .create_no_window()
        .spawn()?;

        let mut std_in = command.take_stdin().ok_or(anyhow!("failed to take std in"))?;


        let time_per_frame = length / frames.len() as f32;

        self.storage.time = 0.0;

        
        for frame in frames {
            self.storage.time += time_per_frame as f64;
            let mut output_frame = RawImage2d::from_raw_rgba(vec![0;frame.len()], (width, height));
            self.run_nodes_on_io_arrays(renderer,  
                RawImage2d::from_raw_rgba(frame, (width, height)), &mut output_frame);
                std_in.write_all(&output_frame.data.to_vec())?;
                // output_data.append(&mut output_frame.data.to_vec());
        }
        command.iter()?.collect_metadata()?;
        
        return Ok(());
    }

    pub fn run_batch(&mut self, renderer: &mut Renderer) {
        let _ = fs::create_dir_all(&self.project_settings.batch_files.save_path);
        let input_path = &self.project_settings.batch_files.files
        [self.project_settings.batch_files.index]
        .path;

        let output_path = self
                .project_settings
                .batch_files
                .save_path
                .join(
                    self.project_settings.batch_files.files
                        [self.project_settings.batch_files.index]
                        .name(),
                )
                .with_extension(
                    self.project_settings.batch_files.files
                        [self.project_settings.batch_files.index]
                        .type_(),
                );
        let a  = self.process_file(renderer, input_path.to_path_buf(), output_path);
        if let Err(e) = a {
            log::error!("{e:?}");
        }
        self.project_settings.batch_files.index += 1;
        if self.project_settings.batch_files.index >= self.project_settings.batch_files.files.len()
        {
            self.project_settings.batch_files.run = false;
            self.project_settings.batch_files.index = 0;
        }
    }

    pub fn render_batch_edit(
        &mut self,
        ui: &Ui,
        sidebar_params: &mut SidebarParams,
        _user_settings: &mut UserSettings,
    ) {
        let size_array = ui.io().display_size;
        let window_size = ImVec2::new(size_array[0], size_array[1]);

        let window_params = vec![
            ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0)),
            ui.push_style_var(imgui::StyleVar::WindowRounding(0.0)),
            // ui.push_style_var(imgui::StyleVar::),
        ];

        ui.window("batch editor").position([sidebar_params.left_sidebar_width, sidebar_params.menu_bar_size[1]], imgui::Condition::Always)
        .size([window_size.x - sidebar_params.left_sidebar_width, window_size.y - sidebar_params.menu_bar_size[1]], imgui::Condition::Always)
        .no_decoration()
        .resizable(false)
        .build(|| {
            
            let color_token = ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.404, 0.0, 1.0]);
            if self.project_settings.generic_io.input_id.is_none() {
                ui.text_wrapped("The generic input node has not been set, you will not be able to perform batch operations");
            }
            if self.project_settings.generic_io.output_id.is_none() {
                ui.text_wrapped("The generic output node has not been set, you will not be able to perform batch operations");
            } 
            if self.project_settings.generic_io.output_id.is_none() || self.project_settings.generic_io.output_id.is_none() {
                ui.text_wrapped("you can set any valid node as a generic input/output using the right click popup menu");
            }

            color_token.end();

            if ui.button("add files") {
                let files = FileDialog::new().pick_files();

                if let Some(files) = files {
                    for f in files {
                        if let Some(file_) = MyFile::new(f) {
                            self.project_settings.batch_files.files.push(file_);
                        }
                    }
                }
                self.project_settings.batch_files.files = self.project_settings.batch_files.files.iter().unique_by(|x| x.path.as_path().to_str()).cloned().collect::<Vec<MyFile>>();
            } 
            ui.same_line();
            if ui.button("add folder") {
                let folder = FileDialog::new().pick_folder();

                if let Some(folder) = folder {
                    let files=recurse_files(folder);
                    if let Ok(files) = files {
                        for f in files {
                            if let Some(file_) = MyFile::new(f) {
                                self.project_settings.batch_files.files.push(file_);
                            }
                        }
                    }
                    self.project_settings.batch_files.files = self.project_settings.batch_files.files.iter().unique_by(|x| x.path.as_os_str()).cloned().collect::<Vec<MyFile>>();
                    
                }
            }
            ui.same_line();
            if ui.button("select output folder") {
                let folder = FileDialog::new().pick_folder();

                if let Some(folder) = folder {
                    self.project_settings.batch_files.save_path = folder;
                }
            }

            ui.text("");

            link_widget(ui, self.project_settings.batch_files.save_path.display().to_string(), self.project_settings.batch_files.save_path.display().to_string());

            ui.spacing();
            ui.spacing();

            
            if ui.button("Name") && self.project_settings.batch_files.files.len()>=2 {
                let first = self.project_settings.batch_files.files[0].path.to_owned();
                let last = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.to_owned();
                self.project_settings.batch_files.files.sort_by_key(|f| f.name());
                let first2 = &self.project_settings.batch_files.files[0].path;
                let last2 = &self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path;
                
                if first2 == &first && &last == last2 {
                    self.project_settings.batch_files.files.reverse();
                }
            }
            
            ui.same_line_with_pos(ui.window_size()[0] / 4.0);

            if ui.button("type") && self.project_settings.batch_files.files.len()>=2 {
                let first = self.project_settings.batch_files.files[0].path.to_owned();
                let last = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.to_owned();
                self.project_settings.batch_files.files.sort_by_key(|f| f.type_());
                let first2 = &self.project_settings.batch_files.files[0].path;
                let last2 = &self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path;

                if first2 == &first && &last == last2 {
                    self.project_settings.batch_files.files.reverse();
                }
            }

            
            ui.same_line_with_pos(ui.window_size()[0] / 2.0);


            if ui.button("Size") && self.project_settings.batch_files.files.len()>=2 {
                let first = self.project_settings.batch_files.files[0].path.to_owned();
                let last = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.to_owned();
                self.project_settings.batch_files.files.sort_by_key(|f| f.size);
                let first2 = &self.project_settings.batch_files.files[0].path;
                let last2 = &self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path;

                if first2 == &first && &last == last2 {
                    self.project_settings.batch_files.files.reverse();
                }
            }

            ui.separator();

            
            ui.child_window("files")
            .build(|| {
                ui.columns(4, "cols", false);
                // ui.set_cursor_pos([ui.cursor_pos()[0], 2.0]);
                
                for file in &self.project_settings.batch_files.files {
                    ui.text(file.name());
                    if ui.is_item_hovered() {
                        ui.tooltip_text(file.name());
                    }
                }
                
                ui.next_column();

                for file in &self.project_settings.batch_files.files {
                    ui.text(file.type_());
                }
                ui.next_column();
                
                let mut f = Formatter::new()
                .scales(Scales::binary())
                .precision(Precision::Significance(3))
                .suffix("B").unwrap();
                for file in &self.project_settings.batch_files.files {
                    ui.text(f.fmt2(file.size));
                    if ui.is_item_hovered() {
                        ui.tooltip_text(file.size.to_string());
                    }
                }
                ui.next_column();
                let mut remove_item = self.project_settings.batch_files.files.len() + 10;
                for i in 0..self.project_settings.batch_files.files.len() {
                    let c_pos = ui.cursor_pos();
                // ui.set_cursor_pos([0.0,c_pos[1]]);
                // ui.invisible_button(file.name(), [ui.window_size()[0], text_height]);
                ui.text_colored([0.0,0.0,0.0,0.3], "remove");
                if ui.is_item_hovered() {
                    ui.set_cursor_pos(c_pos);
                    ui.text_colored([1.0,0.0,0.0,1.0], "remove");
                    if ui.is_item_clicked() {
                        remove_item = i;
                    }
                }
                }

                if remove_item < self.project_settings.batch_files.files.len() {
                self.project_settings.batch_files.files.remove(remove_item);
            }
            });
        });

        for i in window_params {
            i.end();
        }
    }
}
