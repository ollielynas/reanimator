use std::{env::current_exe, fs, time::Instant};

use glium::Program;
use imgui::{sys::{igSetNextWindowSize, ImVec2}, Style, TextureId};
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

use std::sync::Mutex;

// in theoiry this is just a temp solution, but im never going to 
// static DISPLAY_TEXTURE_ID: Mutex<Option<TextureId>> = Mutex::new(None);




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

    let mut return_to_home = false;
    let mut project: Option<Project> = None;
    
    let mut ctx = create_context();
    
    let mut save_timer  = Instant::now();
    
    
    Style::use_light_colors(ctx.style_mut());


    ctx.style_mut().frame_rounding = 2.0;
    ctx.style_mut().frame_border_size = 1.0;
    ctx.style_mut().child_border_size = 1.0;
    ctx.style_mut().popup_border_size = 1.0;
    ctx.style_mut().window_border_size = 1.0;
    ctx.style_mut().window_rounding = 3.0;



    init_with_startup("ReAnimator", |_, _, display| {
    }, move |_, ui, display, renderer| {
        
        if return_to_home {
            project = None;
            return_to_home = false;
        }

        // renderer.textures().get_mut()

        if let Some(ref mut project) = project {
            project.render(ui, &user_settings, renderer);
            return_to_home = project.return_to_home_menu;
            ui.show_default_style_editor();
            if save_timer.elapsed().as_secs_f32() > 5.0 {
                project.save();
                save_timer = Instant::now();
            }

        }else {
            // ReUi::load_and_apply(egui_ctx);
            
            project = Project::project_menu(ui, display, &mut user_settings);
            
        }
    }, None,  &mut ctx);

}