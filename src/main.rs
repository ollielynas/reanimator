use std::{env::current_exe, fs};

use glium::Program;
use imgui::{sys::{igSetNextWindowSize, ImVec2}, Style};
use imgui_winit_support::winit::window;
use platform_dirs::{AppDirs, UserDirs};
use savefile;
use project::Project;
use support::{create_context, init_with_startup};
use user_info::{UserSettings, USER_SETTINGS_SAVEFILE_VERSION};
#[macro_use]
extern crate savefile_derive;


pub mod project;
pub mod node;
pub mod nodes;
pub mod support;
pub mod storage;
pub mod user_info;



fn main() {

    let app_dirs = match AppDirs::new(Some("Reanimator"), false) {
        Some(a) => {
            a.config_dir
        },
        None => {
            current_exe().unwrap()
        },
    };




    let user_dirs = UserDirs::new();

    let mut user_settings: UserSettings = savefile::load_file(app_dirs.join("settings.bat"), USER_SETTINGS_SAVEFILE_VERSION).unwrap_or_default();
    user_settings.update_projects();

    let mut project: Option<Project> = None;
    
    let mut ctx = create_context();
    
    
    
    Style::use_light_colors(ctx.style_mut());

    init_with_startup(file!(), |_, _, display| {
    }, move |_, ui, display| {
        
        if let Some(ref mut project) = project {
            project.render(ui, &user_settings);
            
        }else {
            // ReUi::load_and_apply(egui_ctx);
            
            project = Project::project_menu(ui, display, &mut user_settings);
            
        }
    }, None,  &mut ctx);

}