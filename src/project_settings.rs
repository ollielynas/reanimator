use std::path::PathBuf;

use crate::{batch_edit::RunBatch, generic_io::GenericIO};



#[derive(Savefile)]
pub struct ProjectSettings {
    pub render_ticker: bool,
    pub generic_io: GenericIO,
    pub window_pos: Option<[f32;2]>,
    pub window_size: Option<[f32;2]>,
    pub maximised: bool,
    pub batch_files: RunBatch,
    
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
        }
    }
}