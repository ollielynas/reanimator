use std::{error::Error, ffi::OsStr, fmt::Debug, fs::{self, read_dir}, path::{Path, PathBuf}};

use glium::texture::{RawImage1d, RawImage2d};
use imgui::{sys::ImVec2, Ui};
use imgui_winit_support::winit::error::OsError;
use numfmt::{Formatter, Precision, Scales};
use rfd::FileDialog;
use itertools::Itertools;
use crate::{batch_edit::MyFile, project::Project, sidebar::SidebarParams, storage::Storage, user_info::UserSettings, widgets::link_widget};
use image::{DynamicImage, ImageBuffer, ImageDecoder, Rgba};
use image::{self, ImageFormat};
use image::EncodableLayout;
use imgui::text_filter;
use imgui_glium_renderer::Renderer;





#[derive(Savefile)]
pub struct LocalFiles {
    pub files: Vec<MyFile>,
}


impl LocalFiles {
    pub fn reload(&mut self, storage: &Storage) {

        if !storage.project_root.exists() {
            fs::create_dir_all(&storage.project_root);
        }

        self.files.clear();
        let files=recurse_files(storage.project_root.clone());
                    if let Ok(files) = files {
                        for f in files {
                            if let Some(file_) = MyFile::new(f) {
                                self.files.push(file_);
                            }
                        }
                    }
                    self.files = self.files.iter().unique_by(|x| x.path.clone()).cloned().collect::<Vec<MyFile>>();
    }
}

impl Default for LocalFiles {
    fn default() -> Self {
        Self { files: Default::default() }
    }
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


    pub fn render_local_files(&mut self, ui: &Ui, sidebar_params: &mut SidebarParams, _user_settings: &mut UserSettings) {



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
            if ui.button("add files") {
                let files = FileDialog::new().pick_files();
                if let Some(files) = files {
                    for f in files {
                        
                            let a= fs::copy(f.clone(), self.storage.project_root.join((f.clone()).file_name().unwrap_or(OsStr::new("error"))));
                            println!("{a:?}");
                    }
                }
                self.project_settings.local_files.reload(&self.storage);
            }
            ui.same_line();
            if ui.button("add folder") {
                let folder = FileDialog::new().pick_folder();

                if let Some(folder) = folder {
                    let files=recurse_files(folder);
                    if let Ok(files) = files {
                        for f in files {
                            let a = fs::copy(f.clone(), self.storage.project_root.join((f.clone()).file_name().unwrap_or(OsStr::new("error"))));
                            println!("{a:?}");
                        }
                    }
                }
                self.project_settings.local_files.reload(&self.storage);
            }
            

            ui.text("");

            link_widget(ui, self.storage.project_root.display().to_string(), self.storage.project_root.display().to_string());

            ui.spacing();
            ui.spacing();

            
            if ui.button("Name") && self.project_settings.local_files.files.len()>=2 {
                let first = self.project_settings.local_files.files[0].path.clone();
                let last = self.project_settings.local_files.files[self.project_settings.local_files.files.len() - 1].path.clone();
                self.project_settings.local_files.files.sort_by_key(|f| f.name());
                let first2 = self.project_settings.local_files.files[0].path.clone();
                let last2 = self.project_settings.local_files.files[self.project_settings.local_files.files.len() - 1].path.clone();
                
                if first2 == first && last == last2 {
                    self.project_settings.local_files.files.reverse();
                }
            }
            
            ui.same_line_with_pos(ui.window_size()[0] / 4.0);

            if ui.button("type") && self.project_settings.local_files.files.len()>=2 {
                let first = self.project_settings.local_files.files[0].path.clone();
                let last = self.project_settings.local_files.files[self.project_settings.local_files.files.len() - 1].path.clone();
                self.project_settings.local_files.files.sort_by_key(|f| f.type_());
                let first2 = self.project_settings.local_files.files[0].path.clone();
                let last2 = self.project_settings.local_files.files[self.project_settings.local_files.files.len() - 1].path.clone();

                if first2 == first && last == last2 {
                    self.project_settings.local_files.files.reverse();
                }
            }

            
            ui.same_line_with_pos(ui.window_size()[0] / 2.0);


            if ui.button("Size") && self.project_settings.local_files.files.len()>=2 {
                let first = self.project_settings.local_files.files[0].path.clone();
                let last = self.project_settings.local_files.files[self.project_settings.local_files.files.len() - 1].path.clone();
                self.project_settings.local_files.files.sort_by_key(|f| f.size);
                let first2 = self.project_settings.local_files.files[0].path.clone();
                let last2 = self.project_settings.local_files.files[self.project_settings.local_files.files.len() - 1].path.clone();

                if first2 == first && last == last2 {
                    self.project_settings.local_files.files.reverse();
                }
            }

            ui.separator();

            ui.child_window("files")
            .build(|| {
                ui.columns(4, "cols", false);
                // ui.set_cursor_pos([ui.cursor_pos()[0], 2.0]);
                
                for file in &self.project_settings.local_files.files {
                    ui.text(file.name());
                    if ui.is_item_hovered() {
                        if ui.clipboard_text() != Some(file.path.display().to_string()) {
                            ui.tooltip_text("click to copy path:");
                        } else {
                            ui.tooltip_text("copied path");
                        }
                        ui.tooltip_text(file.path.display().to_string());
                        if ui.is_mouse_clicked(imgui::MouseButton::Left) {
                            ui.set_clipboard_text(file.path.display().to_string());
                        }
                    }
                }

                ui.next_column();

                for file in &self.project_settings.local_files.files {
                    ui.text(file.type_());
                }
                ui.next_column();
                
                let mut f = Formatter::new()
                .scales(Scales::binary())
                .precision(Precision::Significance(3))
                .suffix("B").unwrap();
                for file in &self.project_settings.local_files.files {
                    ui.text(f.fmt2(file.size));
                    if ui.is_item_hovered() {
                        ui.tooltip_text(file.size.to_string());
                    }
                }
                ui.next_column();
                let mut remove_item = self.project_settings.local_files.files.len() + 10;
                for i in 0..self.project_settings.local_files.files.len() {
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

                if remove_item < self.project_settings.local_files.files.len() {
                    let _ = fs::remove_file(&self.project_settings.local_files.files[remove_item].path);
                self.project_settings.local_files.reload(&self.storage);
            }
            });
        });



        for i in window_params {
            i.end();
        }
    }
}