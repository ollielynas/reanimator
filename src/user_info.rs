use std::{
    env::current_exe, fs::{self, DirEntry}, path::PathBuf, sync::{Mutex, MutexGuard}, thread::Thread, time::SystemTime
};

use imgui::{Style, Ui};
use imgui_glium_renderer::Renderer;
use platform_dirs::{AppDirs, UserDirs};
use rfd::FileDialog;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

use glium::Display;


use glium::glutin::surface::WindowSurface;
use std::thread;

use crate::{project::Project, support::create_context};


pub const USER_SETTINGS_SAVEFILE_VERSION: u32 = 2;

#[derive(Savefile, EnumIter, EnumString, PartialEq, Eq, Debug, Clone)]
pub enum UiTheme {
    GenericLightMode,
    GenericDarkMode,
}

impl Default for UiTheme {
    fn default() -> Self {
        UiTheme::GenericLightMode
    }
}

#[derive(Savefile, Clone)]
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
    #[savefile_versions="2.."]
    #[savefile_default_val="false"]
    pub fullscreen: bool,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    loading: bool,
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
            fullscreen: false,
            loading: false,
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

                    ui.checkbox("fullscreen", &mut self.fullscreen);
                    if ui.is_item_hovered() {
                        ui.tooltip_text("This option is a bit buggy at the moment and so is not recommended");
                    }

                }
                tab_bar.end();
            }

        });
    }
}



impl Project {
    
    pub fn project_menu(
        ui: &mut Ui,
        display: &Display<WindowSurface>,
        user_settings: &mut UserSettings,
        renderer: &mut Renderer,
    ) -> Option<Project> {
        let mut new_project = None;
        let size_array = ui.io().display_size;

        let height = size_array[1] * 0.8;
        let pos = [size_array[0] * 0.5, size_array[1] * 0.5];





        // let size = ui.siz
        ui.window("project selector")
            .collapsible(false)
            .movable(false)
            .position_pivot([0.5, 0.5])
            .bring_to_front_on_focus(false)
            .always_auto_resize(true)
            .size([height * 1.5, height], imgui::Condition::Always)
            .position(pos, imgui::Condition::Always)
            .build(|| {
                ui.columns(2, "col 1324", true);
                ui.input_text("##", &mut user_settings.new_project_name)
                    .auto_select_all(true)
                    .build();
                if ui.button("create new project") {
                    new_project = Some(Project::new(
                        user_settings
                            .project_folder_path
                            .join(user_settings.new_project_name.clone()),
                        display.clone(),
                    ));
                }
                ui.next_column();
                // ui.set_window_font_scale(1.2);
                ui.text("load project");
                // ui.set_window_font_scale(1.0);

                ui.window("projects")
                    .always_vertical_scrollbar(true)
                    .position_pivot([0.0, 1.0])
                    // .focused(true)
                    .movable(false)
                    .resizable(false)
                    .collapsible(false)
                    // .draw_background(false)
                    .title_bar(false)
                    .position(
                        [
                            ui.cursor_screen_pos()[0] + 10.0,
                            size_array[1] - height * 0.15,
                        ],
                        imgui::Condition::Always,
                    )
                    .size(
                        [
                            ui.content_region_avail()[0] - 10.0,
                            ui.content_region_avail()[1] - 10.0,
                            ],
                            imgui::Condition::Always,
                        )
                        .build(|| {
                            for project in &user_settings.projects {
                                if ui.button(project.file_name().unwrap().to_str().unwrap()) {
                                    let mut new_project_1 =
                                    Project::new(project.to_path_buf(), display.clone());
                                let _ = new_project_1.save();
                                
                                let mut save_dir = match AppDirs::new(Some("Reanimator"), false) {
                                    Some(a) => {
                                        fs::create_dir_all(a.cache_dir.clone());
                                        a.cache_dir
                                    }
                                    None => current_exe().unwrap(),
                                };

                                let p = save_dir.join(new_project_1.name());
                                
                                fs::create_dir_all(&p);

                                for i in fs::read_dir(p).unwrap() {
                                    if let Ok(i) = i {
                                        if i.metadata()
                                            .unwrap()
                                            .created()
                                            .unwrap()
                                            .elapsed()
                                            .unwrap()
                                            .as_secs()
                                            > (86400 * 7)
                                        {
                                            fs::remove_file(i.path());
                                        }
                                    }
                                }
                                
                                new_project_1.recenter_nodes(ui);
                                
                                new_project = Some(new_project_1);
                            }

                            // ui.same_line_with_spacing(ui.window_size()[0]-ui.calc_text_size("del")[0], 10.0);
                            // ui.push_style_color(imgui::StyleColor::Button, [0.8,0.3,0.3,1.0]);
                            // ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.7,0.3,0.3,1.0]);
                            // ui.push_style_color(imgui::StyleColor::ButtonActive, [0.7,0.4,0.4,1.0]);
                            // ui.button("del");
                        }
                    });

                ui.dummy(ui.content_region_avail());
            });

        // if new_project.is_some() {

        // }

        return new_project;
    }



    pub fn load_objects(&mut self, mut renderer: Mutex<&mut Renderer>) {
        // let mut r = ;
        self.run_nodes(renderer.get_mut().unwrap());
    }

}