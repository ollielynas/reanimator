use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::project::Project;
use crate::user_info::{self, UserSettings};
use comprexor::Extractor;
use comprexor::{CompressionLevel, Compressor};
use platform_dirs::UserDirs;
use rfd::FileDialog;
use system_extensions::dialogues::messagebox::{self, IconType, MessageBox, WindowType};

impl Project {
    pub fn export(&mut self) -> Option<PathBuf> {
        if self.save().is_err() {
            return None;
        };

        let downloads = UserDirs::new().unwrap().download_dir;
        let output_path = FileDialog::new()
            .set_directory(downloads)
            .set_file_name(self.name())
            .add_filter("", &["repj"])
            .set_title("Export Project")
            .set_can_create_directories(true)
            .save_file();

        if let Some(ref output_path) = output_path {
            let out = output_path.display().to_string();
            let input = self.path.display().to_string();
            let compressor = Compressor::new(&input, &out);
            let compress_info = compressor.compress(CompressionLevel::Default);
            log::info!("exported project: {:?}", compress_info);
            let _ = fs::remove_dir_all(output_path.with_extension(""));
        }
        return output_path;
    }
}

pub fn load_project(path: &str, user_settings: &UserSettings) -> Option<PathBuf> {
    let mut user_settings = user_settings.clone();
    user_settings.update_projects();
    let out = user_settings.project_folder_path.display().to_string();
    let extractor = Extractor::new(&path, &out);
    let _extract_info = extractor.extract();

    // let _a = fs::remove_file(path);



    user_settings.update_projects();

    for project in user_settings.projects {
        if project.file_name().is_some() && project.file_name() == PathBuf::from_str(&path).unwrap_or_default().file_name() {
            return Some(project);
        }
    }
    return None;
}
