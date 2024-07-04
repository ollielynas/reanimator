use std::{
    arch::x86_64, env::current_exe, fs::{self, DirEntry}, path::PathBuf, time::SystemTime
};

use platform_dirs::{AppDirs, UserDirs};

pub const USER_SETTINGS_SAVEFILE_VERSION: u32 = 0;

#[derive(Savefile)]
pub struct UserSettings {
    pub new_project_name: String,
    pub project_folder_path: PathBuf,
    pub projects: Vec<PathBuf>,
}

impl Default for UserSettings {
    fn default() -> Self {
        let user_dirs = UserDirs::new();
        

        let project_folder_path = match user_dirs {
            Some(a) => a.document_dir,
            None => current_exe().unwrap(),
        }
        .join("Reanimator");

        println!("{:?}", fs::create_dir_all(project_folder_path.clone()));

        let new = UserSettings {
            new_project_name: "Unnamed Project".to_owned(),
            project_folder_path,
            projects: vec![],
        };

        return new;
    }
}

impl UserSettings {
    pub fn save(&self) {
        let app_dirs = match AppDirs::new(Some("Reanimator"), false) {
            Some(a) => {
                fs::create_dir_all(a.config_dir.clone());
                println!("{:#?}", a);
                a.config_dir
            }
            None => current_exe().unwrap(),
        };

        println!(
            "{:?}",
            savefile::save_file(
                app_dirs.join("settings.bat"),
                USER_SETTINGS_SAVEFILE_VERSION,
                self
            )
        );
    }

    pub fn update_projects(&mut self) {
        let mut projects = fs::read_dir(&self.project_folder_path)
            .unwrap()
            .filter_map(|x| match x {
                Ok(a)
                    if a.metadata().unwrap().is_dir()
                        && fs::metadata(a.path().join("connections.bin")).is_ok() =>
                {
                    Some(a)
                }
                _ => None,
            })
            .collect::<Vec<DirEntry>>();
        
        projects.sort_by(|a, b| {
            b.metadata()
                .unwrap()
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .unwrap()
                        .modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH),
                )
        });

        self.projects = projects.iter().map(|x| x.path()).collect();
    }
}
