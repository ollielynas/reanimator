use std::{error::Error, ffi::OsStr, fmt::Debug, fs::{self, read_dir}, path::{Path, PathBuf}};

use glium::texture::{RawImage1d, RawImage2d};
use imgui::{sys::ImVec2, Ui};
use imgui_winit_support::winit::error::OsError;
use numfmt::{Formatter, Precision, Scales};
use rfd::FileDialog;
use itertools::Itertools;
use crate::{project::Project, sidebar::SidebarParams, user_info::UserSettings, widgets::link_widget};
use image::{DynamicImage, ImageBuffer, ImageDecoder, Rgba};
use image::{self, ImageFormat};
use image::EncodableLayout;
use imgui::text_filter;
use imgui_glium_renderer::Renderer;


#[derive(Savefile, Clone)]
pub struct MyFile {
    pub path: PathBuf,
    pub size: u64, 
}


fn try_load_as_image(path: PathBuf) -> Option<RawImage2d<'static, u8>> {
    let bytes = match fs::read(path) {
        Ok(a) => a,
        Err(e) => {
            println!("{e}");
            return  None;
        }
    };
    let image = match image::load_from_memory(&bytes) {
        Ok(a) => a,
        Err(e) => {
            println!("{e}");
            return None;
        }
    }
    .flipv()
    .into_rgba8();

    let raw_image = RawImage2d::from_raw_rgba(
        image.as_bytes().to_vec(),
        (image.width(), image.height()),
    );

    println!("{:?}", raw_image.format);

    return Some(raw_image);

}

impl MyFile {

    pub fn get_raw(&self) -> Vec<RawImage2d<u8>> {

        let mut output = vec![];

        if let Some(data) = try_load_as_image(self.path.clone()) {
            output.push(
                data
            );
            return output;
        }else {
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
    }else {
        None
    }
    }

    pub fn name(&self) -> String {
        self.path.file_name().unwrap_or(&OsStr::new("Error")).to_str().unwrap().split(".").next().unwrap().to_string()
    }
    pub fn type_(&self) -> String {
        self.path.extension().unwrap_or(&OsStr::new("")).to_str().unwrap().to_string()
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

    pub fn run_batch(&mut self, renderer: &mut Renderer) {


        let binding = self.project_settings.batch_files.files[self.project_settings.batch_files.index].clone();
        let raw_in: Vec<RawImage2d<u8>> = binding.get_raw();
        if raw_in.len() > 0 {
        println!("{}", raw_in[0].width);
        println!("Format {:?}", raw_in[0].format);
        let mut raw_out: RawImage2d<u8> = RawImage2d::from_raw_rgb(vec![], (0,0));

        let raw_in_file = raw_in[0].data.clone();

        self.run_nodes_on_io_arrays(renderer, RawImage2d::from_raw_rgba(raw_in_file.to_vec(), (raw_in[0].width, raw_in[0].height)), &mut raw_out);

        let output_path = self.project_settings.batch_files.save_path.join(self.project_settings.batch_files.files[self.project_settings.batch_files.index].name()).with_extension(self.project_settings.batch_files.files[self.project_settings.batch_files.index].type_());

        let img: ImageBuffer<Rgba<u8>, _> =
                            ImageBuffer::from_raw(raw_out.width, raw_out.height, raw_out.data.into_owned())
                                .unwrap();
                            
                        let img = DynamicImage::ImageRgba8(img).flipv();
        let a = img.save(output_path);
        if a.is_err() {
            println!("{a:?}");
        }
    }
        self.project_settings.batch_files.index += 1;
        if self.project_settings.batch_files.index >= self.project_settings.batch_files.files.len() {
            self.project_settings.batch_files.run = false;
            self.project_settings.batch_files.index = 0;
        }
    }

    pub fn render_batch_edit(&mut self, ui: &Ui, sidebar_params: &mut SidebarParams, _user_settings: &mut UserSettings) {



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
            
            ui.text_wrapped("! only image files are currently supported");
            
            if self.project_settings.generic_io.input_id.is_none() {
                ui.text_wrapped("! The generic inout node has not been set, you will not be able to perform batch operations");
            }
            if self.project_settings.generic_io.output_id.is_none() {
                ui.text_wrapped("! The generic output node has not been set, you will not be able to perform batch operations");
            } 
            if self.project_settings.generic_io.output_id.is_none() || self.project_settings.generic_io.output_id.is_none() {
                ui.text_wrapped("you can set any valid node as a generic input/output using the right click popup menu");
            }

            

            if ui.button("add files") {
                let files = FileDialog::new().pick_files();

                if let Some(files) = files {
                    for f in files {
                        if let Some(file_) = MyFile::new(f) {
                            self.project_settings.batch_files.files.push(file_);
                        }
                    }
                }
                self.project_settings.batch_files.files = self.project_settings.batch_files.files.iter().unique_by(|x| x.path.clone()).cloned().collect::<Vec<MyFile>>();
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
                    self.project_settings.batch_files.files = self.project_settings.batch_files.files.iter().unique_by(|x| x.path.clone()).cloned().collect::<Vec<MyFile>>();
                    
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
                let first = self.project_settings.batch_files.files[0].path.clone();
                let last = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.clone();
                self.project_settings.batch_files.files.sort_by_key(|f| f.name());
                let first2 = self.project_settings.batch_files.files[0].path.clone();
                let last2 = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.clone();
                
                if first2 == first && last == last2 {
                    self.project_settings.batch_files.files.reverse();
                }
            }
            
            ui.same_line_with_pos(ui.window_size()[0] / 4.0);

            if ui.button("type") && self.project_settings.batch_files.files.len()>=2 {
                let first = self.project_settings.batch_files.files[0].path.clone();
                let last = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.clone();
                self.project_settings.batch_files.files.sort_by_key(|f| f.type_());
                let first2 = self.project_settings.batch_files.files[0].path.clone();
                let last2 = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.clone();

                if first2 == first && last == last2 {
                    self.project_settings.batch_files.files.reverse();
                }
            }

            
            ui.same_line_with_pos(ui.window_size()[0] / 2.0);


            if ui.button("Size") && self.project_settings.batch_files.files.len()>=2 {
                let first = self.project_settings.batch_files.files[0].path.clone();
                let last = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.clone();
                self.project_settings.batch_files.files.sort_by_key(|f| f.size);
                let first2 = self.project_settings.batch_files.files[0].path.clone();
                let last2 = self.project_settings.batch_files.files[self.project_settings.batch_files.files.len() - 1].path.clone();

                if first2 == first && last == last2 {
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