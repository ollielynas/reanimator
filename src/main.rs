use std::{borrow::BorrowMut, env::current_exe, fs, time::Instant};

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
pub mod history_tracker;


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





    let mut user_settings: UserSettings = savefile::load_file(app_dirs.join("settings.bat"), USER_SETTINGS_SAVEFILE_VERSION).unwrap_or_default();
    user_settings.update_projects();

    let mut return_to_home = false;
    let mut project: Option<Project> = None;
    
    let mut ctx: imgui::Context = create_context();
    
    let mut save_timer  = Instant::now();

    let mut settings_window_open = false;
    
    
    user_settings.load_theme(&mut ctx);



    init_with_startup("ReAnimator", |_, _, display| {
    }, move |_, ui, display, renderer| {

        

        
        if return_to_home {
            project = None;
            return_to_home = false;
        }

        let size_array = ui.io().display_size;

        // if let Some(a) = ui.window("background")
        // .bring_to_front_on_focus(false)
        // .no_inputs()
        // .position([0.0,0.0], imgui::Condition::Always)
        // .focused(false)
        // .size(size_array, imgui::Condition::Always)
        // .no_decoration()
        // .no_nav()
        // .begin() {
            
        //     a.end();
        // }

        // ctx.io_mut();
        // ctx.new_frame()
        // renderer.textures().get_mut()

        if let Some(ref mut project) = project {
            project.render(ui, &user_settings, renderer);
            return_to_home = project.return_to_home_menu;
            // ui.show_default_style_editor();
            if save_timer.elapsed().as_secs_f32() > 2.0 {
                save_timer = Instant::now();
                if user_settings.history {
                let r = project.update_history_and_save();
                match r {
                    Ok(_) => {},
                    Err(e) => {println!("{e}")},
                }
                }else {
                    project.save();
                }
            }

        }else {
            // ReUi::load_and_apply(egui_ctx);
            
            project = Project::project_menu(ui, display, &mut user_settings);
            ui.window("settings button")
        .draw_background(false)
        .movable(false)
        .no_decoration()
        .position_pivot([1.0,0.0])
        .size_constraints([ui.calc_text_size("settings xxxxxxxx")[0],-1.0], [9999.0,-1.0])
        .position([size_array[0], 0.0], imgui::Condition::Always)
        .build(|| {
            if ui.button("settings") {
                settings_window_open = true;
            }
        });
        }

        if settings_window_open {
            user_settings.settings_window(ui, &mut settings_window_open);
        }


    }, None,  &mut ctx);

}