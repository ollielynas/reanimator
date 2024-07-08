use std::{
    arch::x86_64, env::current_exe, fs::{self, DirEntry}, path::PathBuf, time::SystemTime
};

use imgui::{ConfigFlags, Style, Ui};
use platform_dirs::{AppDirs, UserDirs};
use rfd::FileDialog;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

pub const USER_SETTINGS_SAVEFILE_VERSION: u32 = 1;

#[derive(Savefile, EnumIter, EnumString, PartialEq, Eq, Debug)]
pub enum UiTheme {
    GenericLightMode,
    GenericDarkMode,
}

impl Default for UiTheme {
    fn default() -> Self {
        UiTheme::GenericLightMode
    }
}

#[derive(Savefile)]
pub struct UserSettings {
    pub new_project_name: String,
    pub project_folder_path: PathBuf,
    pub projects: Vec<PathBuf>,
    #[savefile_versions="1.."]
    #[savefile_default_val="false"]
    pub history: bool,
    #[savefile_versions="1.."]
    pub ui_theme: UiTheme,
    global_font_scale: f32,
    scroll_to_scale: bool,
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
            history: false,
            ui_theme: UiTheme::default(),
            global_font_scale: 1.2,
            scroll_to_scale: false,
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

    pub fn load_theme(&self, ctx: &mut imgui::Context) {

        ctx.io_mut().font_global_scale = self.global_font_scale;

        ctx.io_mut().font_allow_user_scaling = true;
        
        // ctx.load_ini_settings(data)
        

        match self.ui_theme {
            UiTheme::GenericLightMode | UiTheme::GenericDarkMode => {
                ctx.style_mut().frame_rounding = 2.0;
                ctx.style_mut().frame_border_size = 1.0;
                ctx.style_mut().child_border_size = 1.0;
                ctx.style_mut().popup_border_size = 1.0;
                ctx.style_mut().window_border_size = 1.0;
                ctx.style_mut().window_rounding = 3.0;
            }
        }


        match self.ui_theme {
            UiTheme::GenericLightMode => {
                Style::use_light_colors(ctx.style_mut());
            },
            UiTheme::GenericDarkMode => {
                Style::use_dark_colors(ctx.style_mut());
            },
        }
    }

    pub fn settings_window(&mut self, ui: &Ui, window_open: &mut bool) {
        // println!("settings1");
        let screen_size_array = ui.io().display_size;

        ui.window("settings")
        .no_decoration()
        .resizable(false)
        .collapsible(false)
        .position([0.0,0.0], imgui::Condition::Always)
        .size(screen_size_array, imgui::Condition::Always)
        .build(||{
            ui.set_window_font_scale(1.3);
            ui.text("settings");
            ui.set_window_font_scale(1.0);
            ui.same_line();
            ui.spacing();
            ui.same_line();
            if ui.small_button("save and close") {
                self.save();
                *window_open = false;
            }
            ui.spacing();
            ui.spacing();
            ui.spacing();

            if let Some(tab_bar)=ui.tab_bar("settings tab bar") {
                if let Some(_general_settings) = ui.tab_item("general") {
                    ui.spacing();
                    ui.spacing();
                    if ui.button("change project folder") {
                        let new_folder = FileDialog::new()
                            .set_directory(&self.project_folder_path)
                            .set_can_create_directories(true)
                            .set_title("set project folder")
                            .pick_folder();
    
                        if new_folder.is_some() {
                            self.project_folder_path = new_folder.unwrap();
                            self.update_projects();
                            self.save();
                        }
                    }

                    ui.checkbox("save snapshots", &mut self.history);
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Saves periodic snapshots of your project when changes are made.\n These snapshots can then be reloaded at any point");
                    }
                }
                if let Some(_ui_settings) = ui.tab_item("ui") {
                    // ui.push_style_var()
                    ui.spacing();
                    ui.spacing();
                    ui.text_wrapped("In order for some of these changes to have an effect the application must be re-launched");
                    ui.spacing();
                    // ui.style().scale_all_sizes(scale_factor)
                    if let Some((mut current_item, _)) = UiTheme::iter().enumerate().find(|(_,x)| x==&self.ui_theme) {
                        let before = current_item;
                        ui.combo("Theme", &mut current_item, &UiTheme::iter().collect::<Vec<UiTheme>>(), |x: &UiTheme| {format!("{:?}",x).into()});
                        if before != current_item {
                            self.ui_theme = UiTheme::iter().nth(current_item).unwrap();
                        }
                    }
                    ui.input_float("global font scale", &mut self.global_font_scale)
                    .no_horizontal_scroll(false)
                    .build();
                    self.global_font_scale = self.global_font_scale.clamp(0.2, 5.0);
                    ui.indent_by(20.0);
                    ui.set_window_font_scale(self.global_font_scale / ui.io().font_global_scale);
                    ui.text("Font Scale Preview");
                    ui.set_window_font_scale(1.0);
                    ui.indent_by(-20.0);

                    ui.checkbox("CTRL + Mousewheel font scale", &mut self.scroll_to_scale);
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Use CTRL + Mousewheel to change the \n font size of an individual window");
                    }

                    let fonts = ui.fonts().fonts();
                    if false {
                    if let Some((mut current_font_index, _current_font_id)) = fonts.iter().enumerate().find(|(_,x)| **x==ui.current_font().id()) {
                        let index_copy = current_font_index;
                        ui.combo("Font", &mut current_font_index, &fonts, |f: &imgui::FontId| {
                            format!("{f:?}").into()
                        });
                        if index_copy != current_font_index {
                            
                            ui.push_font(fonts[current_font_index]).end();
                        }
                    }}

                }
                tab_bar.end();
            }

        });
    }
}
