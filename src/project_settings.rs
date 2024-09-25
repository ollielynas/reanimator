use std::path::PathBuf;

use crate::{batch_edit::RunBatch, generic_io::GenericIO, project_files::LocalFiles};


pub const PROJECT_SETTINGS_VERSION: u32 = 0;

#[derive(Savefile)]
pub struct ProjectSettings {
    pub render_ticker: bool,
    ///this is where the "generic input" and "generic output" nodes set by the user can be accessed
    pub generic_io: GenericIO,
    pub window_pos: Option<[f32;2]>,
    pub window_size: Option<[f32;2]>,
    /// this feature is not currently implemented

    pub maximised: bool,
    pub batch_files: RunBatch,
    #[savefile_ignore]
    pub local_files: LocalFiles,
}


impl Default for ProjectSettings {


    
    fn default() -> Self {
        ProjectSettings {
            render_ticker: false,
            generic_io: GenericIO::default(),
            window_pos: None,
            window_size: None,
            maximised: true,
            batch_files: RunBatch {
                files: vec![],
                save_path: PathBuf::new(),
                run: false,
                index: 0,
            },
            local_files: LocalFiles::default(),
        }
    }
}