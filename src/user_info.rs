use std::{
    collections::HashMap,
    env::current_exe,
    fs::{self, DirEntry},
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    thread::Thread,
    time::SystemTime,
};

use imgui::{FontConfig, FontSource, Style, Ui};
use imgui_glium_renderer::Renderer;
use platform_dirs::{AppDirs, UserDirs};
use rfd::FileDialog;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

use glium::Display;

use log::*;

use glium::glutin::surface::WindowSurface;
use std::thread;
use win_msgbox::Okay;

use crate::{
    fonts::MyFonts, popups::set_as_default_for_filetype, project::Project, relaunch_program,
    support::FONT_SIZE,
};

pub const USER_SETTINGS_SAVEFILE_VERSION: u32 = 6;

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

fn none_val_font_id() -> Option<imgui::FontId> {
    None
}

#[derive(Savefile, Clone)]
pub struct UserSettings {
    pub new_project_name: String,
    pub project_folder_path: PathBuf,
    pub projects: Vec<PathBuf>,
    #[savefile_versions = "1.."]
    #[savefile_default_val = "false"]
    pub history: bool,
    #[savefile_versions = "1.."]
    pub ui_theme: UiTheme,
    global_font_scale: f32,
    scroll_to_scale: bool,
    #[savefile_versions = "2.."]
    #[savefile_default_val = "false"]
    pub fullscreen: bool,
    #[savefile_versions = "3.."]
    #[savefile_default_val = "120"]
    pub max_fps: i32,
    #[savefile_versions = "4.."]
    #[savefile_default_val = "Default"]
    pub font: String,
    #[savefile_versions = "5.."]

    /// doesn't currently work, and should not be used
    pub node_speed: HashMap<String, Vec<f32>>,
    #[savefile_versions = "5.."]
    #[savefile_default_val = "true"]
    pub dots: bool,
    #[savefile_default_fn = "none_val_font_id"]
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub font_id: Option<imgui::FontId>,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    loading: bool,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    selected_project: Option<PathBuf>,
    #[savefile_versions = "6.."]
    pub finished_setup: bool,
    #[savefile_versions = "6.."]
    pub install_ffmpeg: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        let user_dirs = UserDirs::new();

        let project_folder_path = match user_dirs {
            Some(a) => a.document_dir,
            None => current_exe().unwrap(),
        }
        .join("ReAnimator");

        log::info!("{:?}", fs::create_dir_all(project_folder_path.clone()));

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
            max_fps: 120,
            font: "Default".to_owned(),
            font_id: None,
            dots: true,
            finished_setup: false,
            install_ffmpeg: true,
            node_speed: savefile::load_from_mem::<HashMap<String, Vec<f32>>>(
                include_bytes!("node_speeds.bin"),
                0,
            )
            .unwrap_or_default(),
            selected_project: None,
        };

        return new;
    }
}

impl UserSettings {
    pub fn save(&self) {
        let app_dirs = match AppDirs::new(Some("ReAnimator"), false) {
            Some(a) => {
                let _ = fs::create_dir_all(a.config_dir.clone());
                log::info!("{:#?}", a);
                a.config_dir
            }
            None => current_exe().unwrap(),
        };

        log::info!(
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

    pub fn load_theme(&mut self, ctx: &mut imgui::Context) {
        ctx.io_mut().font_global_scale = self.global_font_scale;

        ctx.io_mut().font_allow_user_scaling = self.scroll_to_scale;

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

        let fonts = MyFonts::new();

        if self.font != "Default" {
            if let Ok(handle) = fonts.fonts.select_family_by_name(&self.font) {
                let mut font = None;
                for h in handle.fonts() {
                    let mut fonts = vec![];
                    match h.load() {
                        Ok(a) => {
                            // log::info!("{:?}",a.full_name());
                            fonts.push(a);
                        }
                        Err(_) => {}
                    }
                    fonts.sort_by_key(|x| {
                        x.full_name()
                            .to_lowercase()
                            .replace("bold", "bolddddddddddddddd")
                            .replace("regular", "")
                            .len()
                    });
                    if fonts.len() > 0 {
                        font = Some(fonts[0].clone());
                    }
                    log::info!("{font:?}");
                }
                if let Some(font) = font {
                    if let Some(data) = font.copy_font_data() {
                        log::info!("added font");
                        let id: imgui::FontId = ctx.fonts().add_font(&[FontSource::TtfData {
                            data: &data,
                            size_pixels: FONT_SIZE,
                            config: Some(FontConfig {
                                // As imgui-glium-renderer isn't gamma-correct with
                                // it's font rendering, we apply an arbitrary
                                // multiplier to make the font a bit "heavier". With
                                // default imgui-glow-renderer this is unnecessary.
                                // rasterizer_multiply: 1.5,
                                // Oversampling font helps improve text rendering at
                                // expense of larger font atlas texture.
                                oversample_h: 4,
                                oversample_v: 4,
                                ..FontConfig::default()
                            }),
                        }]);

                        self.font_id = Some(id);

                        if !ctx.fonts().is_built() {
                            log::info!("font not build");
                            ctx.fonts().build_rgba32_texture();
                            ctx.fonts().build_alpha8_texture();
                            if !ctx.fonts().is_built() {
                                self.font = "Default".to_owned();
                                self.save();
                                // #[cfg(all(target_os="windows", not(debug_assertions)))]{
                                win_msgbox::show::<Okay>(&format!("Font Loading Error"));
                                // }
                                let id: imgui::FontId =
                                    ctx.fonts().add_font(&[FontSource::TtfData {
                                        data: include_bytes!(
                                            "support/resources/WorkSans-VariableFont_wght.ttf"
                                        ),
                                        size_pixels: FONT_SIZE,
                                        config: Some(FontConfig {
                                            // As imgui-glium-renderer isn't gamma-correct with
                                            // it's font rendering, we apply an arbitrary
                                            // multiplier to make the font a bit "heavier". With
                                            // default imgui-glow-renderer this is unnecessary.
                                            // rasterizer_multiply: 1.5,
                                            // Oversampling font helps improve text rendering at
                                            // expense of larger font atlas texture.
                                            oversample_h: 4,
                                            oversample_v: 4,
                                            ..FontConfig::default()
                                        }),
                                    }]);

                                self.font_id = Some(id);

                                ctx.fonts().build_rgba32_texture();
                                ctx.fonts().build_alpha8_texture();
                            }
                            // ctx.new_frame();
                        }
                    }
                };
            }
        }

        match self.ui_theme {
            UiTheme::GenericLightMode => {
                Style::use_light_colors(ctx.style_mut());
            }
            UiTheme::GenericDarkMode => {
                Style::use_dark_colors(ctx.style_mut());
            }
        }
    }

    pub fn settings_window(&mut self, ui: &Ui, window_open: &mut bool, fonts: &MyFonts) {
        // log::info!("settings1");
        let screen_size_array = ui.io().display_size;

        ui.window("settings")
        .no_decoration()
        .resizable(false)
        .collapsible(false)
        .position([0.0,0.0], imgui::Condition::Always)
        .size(screen_size_array, imgui::Condition::Always)
        .build(||{
            ui.columns(2, "settinsg col", false);
            
            ui.set_window_font_scale(1.7);
            ui.text("settings");
            ui.set_window_font_scale(1.0);
            ui.spacing();
            // ui.same_line();
            ui.spacing();
            // ui.same_line();
            if ui.button("save and close") {
                self.save();
                *window_open = false;
            }
            // ui.same_line();
            ui.spacing();
            // ui.same_line();
            if ui.button("save and relaunch") {
                self.save();
                relaunch_program(false, "");
            }
            ui.set_column_width(0, ui.window_size()[0] * 0.25);


            ui.next_column();


            ui.child_window("settings child window")
            .size([0.0,0.0])
            .border(true)
            .build(|| {

            if let Some(tab_bar)=ui.tab_bar("settings tab bar") {
                ui.indent();
                // ui.indent();
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


                    ui.input_int("target fps", &mut self.max_fps).build();
                    self.max_fps = self.max_fps.max(10);
                    // if ui.is_item_hovered() {
                    //     ui.tooltip_text("The programme kinda struggles to hit this atm often going 20 fps or so over the target");
                    // }

                }
                if let Some(_general_settings) = ui.tab_item("ui") {
                    ui.spacing();
                    ui.spacing();

                    ui.checkbox("grid pattern", &mut self.dots);

                }
                if let Some(_ui_settings) = ui.tab_item("ui (reboot required)") {
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
                    
                    let mut current_font = match fonts.font_names.iter().enumerate().find(|(i,x)|  x == &&self.font) {
                        Some((i,v)) => i + 1,
                        None => 0,
                    };

                    let before = current_font;
                    

                    ui.combo_simple_string("custom font", &mut current_font, &[vec!["Default".to_owned()], fonts.font_names.clone()].concat());


                    if before != current_font {
                        self.font = [vec!["Default".to_owned()], fonts.font_names.clone()].concat()[current_font].clone();
                    }


                    ui.checkbox("fullscreen", &mut self.fullscreen);
                    if ui.is_item_hovered() {
                        ui.tooltip_text("This option is a bit buggy at the moment and so is not recommended");
                    }

                }
                if let Some(_general_settings) = ui.tab_item("advanced") {
                    ui.spacing();
                    ui.spacing();
                    if ui.button("Register as default application for .repj files") {
                        set_as_default_for_filetype();
                    }
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Application must be run as admin");
                    }
                    if ui.button("redo setup") {
                        self.finished_setup = false;
                        self.save();
                        relaunch_program(false, "");
                    }
                }
                tab_bar.end();
            }

        });
    });
    }
}

impl Project {
    pub fn project_menu(
        ui: &Ui,
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
                                // let _ = new_project_1.save();

                                // I have no idea what this code does.
                                let save_dir = match AppDirs::new(Some("ReAnimator"), false) {
                                    Some(a) => {
                                        fs::create_dir_all(a.cache_dir.clone());
                                        log::info!("cache_dir{:?}", a.cache_dir.clone());
                                        a.cache_dir
                                    }
                                    None => current_exe().unwrap(),
                                };

                                let p = save_dir.join(new_project_1.name());

                                let _ = fs::create_dir_all(&p);

                                for i in fs::read_dir(&p).unwrap() {
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
                                            let _ = fs::remove_file(i.path());
                                        }
                                    }
                                }

                                fs::remove_dir_all(&p);

                                new_project_1.recenter_nodes(ui);

                                new_project = Some(new_project_1);
                            }

                            let item_size = ui.item_rect_size();

                            if (ui.is_window_hovered()
                                && (0.0..item_size[1])
                                    .contains(&(ui.cursor_screen_pos()[1] - ui.io().mouse_pos[1])))
                                || user_settings.selected_project.as_ref() == Some(project)
                            {
                                ui.same_line();

                                if ui.button("...") {
                                    user_settings.selected_project = Some(project.clone());
                                }
                                if ui.is_item_clicked() {
                                    user_settings.selected_project = Some(project.clone());
                                }
                            };
                        }
                    });

                ui.dummy(ui.content_region_avail());

                if ui.is_window_hovered() {
                    ui.close_current_popup();
                }
            });

        // if new_project.is_some() {

        // }

        let mut deleted_project = false;
        let mut closed_popup = false;

        if let Some(project_path) = &user_settings.selected_project {
            ui.open_popup("project_options");

            ui.popup("project_options", || {
                if ui.button("trash") {
                    log::info!("deleted project {:?}", trash::delete(project_path));
                    deleted_project = true;
                }

                if ui.button("export") {
                    log::info!("exporting project");

                    let mut new_project_1 =
                        Project::new(project_path.to_path_buf(), display.clone());

                    let export_path = new_project_1.export();

                    if let Some(export_path) = export_path {
                        if let Some(parent_path) = export_path.parent() {
                            open::that_detached(parent_path);
                        }
                    };
                    closed_popup = true;
                }

                if !ui.is_window_hovered()
                    && ui.is_any_mouse_down()
                    && !ui.is_any_item_active()
                    && (ui.mouse_pos_on_opening_current_popup()[0] - ui.io().mouse_pos[0]).abs()
                        > 1.0
                    && (ui.mouse_pos_on_opening_current_popup()[1] - ui.io().mouse_pos[1]).abs()
                        > 1.0
                {
                    closed_popup = true;
                }
            });

            if deleted_project {
                user_settings.update_projects();
            }

            if deleted_project || closed_popup {
                user_settings.selected_project = None;
            }
        }

        return new_project;
    }

    pub fn load_objects(&mut self, mut renderer: Mutex<&mut Renderer>) {
        // let mut r = ;
        self.run_nodes(renderer.get_mut().unwrap());
    }
}
