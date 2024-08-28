use std::fs;
use std::path::PathBuf;

use crate::project::Project;
use crate::user_info::{self, UserSettings};
use comprexor::Extractor;
use comprexor::{CompressionLevel, Compressor};
use platform_dirs::UserDirs;
use rfd::FileDialog;
use system_extensions::dialogues::messagebox::{self, IconType, MessageBox, WindowType};

impl Project {
    pub fn export(&mut self) {
        if self.save().is_err() {
            return;
        };

        let downloads = UserDirs::new().unwrap().download_dir;
        let output_path = FileDialog::new()
            .set_directory(downloads)
            .set_file_name(self.name())
            .add_filter("", &["repj"])
            .set_title("Export Project")
            .set_can_create_directories(true)
            .save_file();

        if let Some(output_path) = output_path {
            let out = output_path.display().to_string();
            let input = self.path.display().to_string();
            let compressor = Compressor::new(&input, &out);
            let compress_info = compressor.compress(CompressionLevel::Default);
        }
    }
}

pub fn load_project(path: String, mut user_settings: UserSettings) -> Option<PathBuf> {
    user_settings.update_projects();
    let projects = user_settings.projects.clone();
    let out = user_settings.project_folder_path.display().to_string();
    let extractor = Extractor::new(&path, &out);
    let extract_info = extractor.extract();

    let _a = fs::remove_file(path);

    user_settings.update_projects();

    let mut new_projects = user_settings.projects;
    
    new_projects.retain(|x| !projects.contains(x));

    if extract_info.is_ok() && new_projects.len() > 0 {
            return Some(new_projects[0].clone());
    }
    return None;
}
