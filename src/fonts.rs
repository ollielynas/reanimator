use std::{collections::HashSet, path::PathBuf};

use font_kit::{self, loader::Loader, source::Source, sources::multi::MultiSource};
use imgui::Ui;

use crate::storage::Storage;

pub struct MyFonts {
    pub fonts: MultiSource,
    pub font_names: Vec<String>,
}

impl MyFonts {
    pub fn new() -> MyFonts {
        let mut f = MyFonts {
            fonts: MultiSource::from_sources(vec![
            // Box::new(font_kit::source::)
            ]),
            font_names: vec![],
        };

        f.load_fonts();

        return f;
    }

    pub fn load_fonts(&mut self) {
        // let fonts = font_kit::sources::multi::MultiSource::
        // all_fonts(&self)
        let sys = font_kit::source::SystemSource::new();
        // log::info!("{:?}",sys.all_families());
        // let sys = font_kit::source::SystemSource;

        self.font_names = sys.all_families().unwrap_or_default();
        self.font_names.sort();
        let multi: MultiSource = MultiSource::from_sources(vec![
            Box::new(sys),
            // Box::new(font_kit::source::)
        ]);

        // for a in self.fonts.iter() {
        //     while let Ok(b) = a.all_fonts() {
        //         for c in b {
        //             if let Ok(font) = c.load() {
        //                 log::info!("found font");
        //                 if let Some(name) = font.postscript_name() {
        //                     self.font_names.push(name);
        //                 }
        //             };
        //         }
        //     }
        // }

        self.fonts = multi;
    }
}
