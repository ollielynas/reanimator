use std::{
    env::current_exe,
    ffi::OsStr,
    fs::{self},
    io::Error,
    time::UNIX_EPOCH,
};

use crate::project::Project;
use comprexor::{CompressionLevel, Compressor, Extractor};
use dtt::{self, DateTime};
use imgui::Ui;

use platform_dirs::AppDirs;
use savefile::{self};
use std::path::PathBuf;
// #[derive(Hash)]

use blake2::{Blake2s256, Digest};


#[derive(Savefile, PartialEq)]
pub struct Snapshot {
    description: String,
    path: PathBuf,
    time: String,
    index: f64,
}

impl Project {
    pub fn update_history_and_save(&mut self) -> Result<(), Error> {
        let mut old = Project::new(self.path.clone(), self.storage.display.clone());
        let a = self.save();

        match a {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::other(format!("a {e:?}")));
            }
        }

        let mut description = String::new();

        if self.nodes.len() != old.nodes.len() {
            if self.nodes.len() > old.nodes.len() {
                description = format!("added {} node/s", self.nodes.len() - old.nodes.len());
            }else {
                description = format!("removed {} node/s", old.nodes.len() - self.nodes.len());
            }
        } else {
            if old.connections != self.connections {
                description = "changed connection".to_owned()
            }
        }

        if description == String::new() && self.snapshots != vec![] {
            return Ok(());
        }

        let mut save_dir = match AppDirs::new(Some("Reanimator"), false) {
            Some(a) => {
                fs::create_dir_all(a.cache_dir.clone());
                a.cache_dir
            }
            None => current_exe().unwrap(),
        };

        old.path = save_dir.join(self.name()).join(self.name());

        let o = old.save();

        match o {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::other(format!("o {e:?}")));
            }
        }

        let mut hash = Blake2s256::new();
        let hash_str = file_hashing::get_hash_folder(self.path.clone(), &mut hash, 12, |_| {})
            .unwrap_or("hash error".to_owned());

        let new_path = save_dir.join(self.name()).join(hash_str);

        let mut snapshot = Snapshot {
            description: description,
            path: new_path.clone(),
            time: DateTime::new().format("%d/%m/%Y %H:%M"),
            index: UNIX_EPOCH.elapsed().unwrap().as_secs_f64(),
        };

        let compressor: Compressor = Compressor::new(
            old.path.as_os_str().to_str().unwrap(),
            new_path.as_os_str().to_str().unwrap(),
        );
        let _compress_info = compressor.compress(CompressionLevel::Fast)?;

        let _its_ok_if_this_errors = fs::remove_dir_all(old.path.clone());

        // if fs::metadata(new_path.clone()).is_ok() {
        //     return Ok(());
        // }else {
        // };

        let a = savefile::save_file(new_path.with_extension("snapshot"), 0, &snapshot);

        match a {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::other(format!("snapshot file {e:?}")));
            }
        }

        self.snapshots = vec![];

        for file in fs::read_dir(match new_path.parent() {
            Some(a) => a,
            None => {
                return Err(Error::other("no parent"));
            }
        })? {
            if let Ok(file) = file {
                if file.path().extension() == Some(OsStr::new("snapshot")) {
                    if let Ok(a) = savefile::load_file(file.path(), 0) {
                        self.snapshots.push(a)
                    };
                }
            }
        }

        self.snapshots.sort_by(|a, b| b.index.total_cmp(&a.index));

        return Ok(());
    }

    pub fn history_window(&mut self, ui: &Ui) {
        ui.window("timeline")
            .always_vertical_scrollbar(true)
            .opened(&mut self.display_history)
            .build(|| {
                if self.snapshots.len() > 0 {
                    if ui.button("load past state") {
                        let extractor = Extractor::new(
                            self.snapshots[self.selected_snapshot as usize]
                                .path
                                .as_os_str()
                                .to_str()
                                .unwrap(),
                            self.path.parent().unwrap().as_os_str().to_str().unwrap(),
                        );

                        log::info!("extractor {:?}", extractor.extract());
                        let new = Project::new(self.path.clone(), self.storage.display.clone());

                        self.nodes = new.nodes;
                        self.connections = new.connections;
                    }

                    let mut options = vec![];

                    for snapshot in &self.snapshots {
                        let time = snapshot
                            .time
                            .replace(&DateTime::new().format("%d/%m/%Y"), "");
                        options.push(format!("{} : {}", time, snapshot.description));
                    }
                    let item_width = ui.push_item_width(-1.0);
                    ui.list_box(
                        "##",
                        &mut self.selected_snapshot,
                        &options.iter().map(|x| x).collect::<Vec<&String>>(),
                        (ui.content_region_avail()[1] / (ui.calc_text_size("x")[1] + 5.0)) as i32,
                    );
                    item_width.end();
                }
            });
    }
}
