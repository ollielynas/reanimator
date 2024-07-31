#![cfg_attr(
    all(
      target_os = "windows",
      not(debug_assertions),
    ),
    windows_subsystem = "windows"
  )]

use std::{borrow::BorrowMut, env::current_exe, fs, process::exit, thread::{sleep, sleep_ms}, time::{Duration, Instant}};


use fonts::MyFonts;
use imgui_winit_support::winit::{error::OsError, monitor::VideoMode, window::{self, Fullscreen}};
use platform_dirs::{AppDirs, UserDirs};
use savefile;
use project::Project;
use self_update::cargo_crate_version;
use support::{create_context, init_with_startup};
use user_info::{UserSettings, USER_SETTINGS_SAVEFILE_VERSION};
use win_msgbox::{raw::w, Okay};
#[macro_use]
extern crate savefile_derive;


pub mod project;
pub mod node;
pub mod nodes;
pub mod support;
pub mod storage;
pub mod user_info;
pub mod history_tracker;
pub mod advanced_color_picker;
pub mod widgets;
pub mod fonts;
pub mod render_nodes;





// in theoiry this is just a temp solution, but im never going to 
// static DISPLAY_TEXTURE_ID: Mutex<Option<TextureId>> = Mutex::new(None);


fn update() -> Result<(), Box<dyn (::std::error::Error)>> {
    println!("updating");
    
    let relase_builds = self_update::backends::github::ReleaseList::configure()
    .repo_owner("ollielynas")
    .repo_name("reanimator").build();


    let status = self_update::backends::github::Update::configure()
    .repo_owner("ollielynas")
    .repo_name("reanimator")
        // .identifier(".zip")
        .bin_name("reanimator")
        // .bin_path_in_archive()
        .no_confirm(false)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    
    if status.updated() {
        #[cfg(all(target_os="windows", not(debug_assertions)))]{
            win_msgbox::show::<Okay>(&format!("Updated to version: {}", status.version()));
            relaunch_windows(false);
        }
        exit(0);
    }
    // self_update
    println!("Update status: `{}`!", status.version());
    
    Ok(())
}


fn main() {

    #[cfg(all(target_os="windows", not(debug_assertions)))]{



    std::panic::set_hook(Box::new(|a| {
        win_msgbox::show::<Okay>(&format!("Program Crashed \n {a}"));
    }));

        let a = update();

        if let Err(a2) = a {

            if a2.to_string().contains("os error 5") {
                if win_msgbox::show::<Okay>(&format!("New update avalible, press ok to install")).is_ok() {
                    relaunch_windows(true);
                };
            }else {
                win_msgbox::show::<Okay>(&format!("Error Updating \n {a2}"));
            }
            // panic!("{:?}", a2);
        }
    }
    // panic!("test");

    // println!("{}");

    // println!("test");

    // update();

    let app_dirs = match AppDirs::new(Some("Reanimator"), false) {
        Some(a) => {
            a.config_dir
        },
        None => {
            current_exe().unwrap()
        },
    };


    let mut fonts = MyFonts::new();


    let mut user_settings: UserSettings = savefile::load_file(app_dirs.join("settings.bat"), USER_SETTINGS_SAVEFILE_VERSION).unwrap_or_default();
    user_settings.update_projects();

    let mut return_to_home = false;
    let mut project: Option<Project> = None;
    
    let mut ctx: imgui::Context = create_context();

    
    let mut save_timer  = Instant::now();

    let mut settings_window_open = false;
    
    
    user_settings.load_theme(&mut ctx);

    let fullscreen = user_settings.fullscreen; 



    init_with_startup("ReAnimator", |_, _, display| {
    }, move |_, ui, display, renderer, drop_file| {
        
        let mut global_font_tokens = vec![];
        if let Some(font_id) = user_settings.font_id {
        if ui.fonts().get_font(font_id).is_some() {
            global_font_tokens.push(ui.push_font(font_id));
        }
        }

        if return_to_home {
            project = None;
            return_to_home = false;
        }

        let size_array = ui.io().display_size;



        if let Some(ref mut project) = project {


            if let Some(path) = drop_file {
                project.drop_file(path, ui);
            } 
            
            project.render(ui, &mut user_settings, renderer);
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
                    let _ = project.save();
                }
            }

        }else {
            
            project = Project::project_menu(ui, display, &mut user_settings, renderer);
            ui.window("settings button")
        .draw_background(false)
        .movable(false)
        .no_decoration()
        .position_pivot([0.0,1.0])
        .size_constraints([ui.calc_text_size("settings xxxxxxxx")[0],-1.0], [9999.0,-1.0])
        .position([10.0,size_array[1] + 5.0], imgui::Condition::Always)
        .build(|| {
            if ui.button("settings") {
                settings_window_open = true;
            }
            // ui.same_line();
        });
            ui.window("version")
        .draw_background(false)
        .movable(false)
        .no_decoration()
        .position_pivot([0.5,1.0])
        .size_constraints([ui.calc_text_size("xxxxxxxxxxxxx")[0],-1.0], [9999.0,-1.0])
        .position([size_array[0]*0.5,size_array[1]], imgui::Condition::Always)
        .build(|| {
            ui.text("v".to_owned()+cargo_crate_version!());
            // cargo_crate_version!()
        });
        }



        if settings_window_open {
            user_settings.settings_window(ui, &mut settings_window_open, &fonts);
        }
    
        sleep( Duration::from_secs_f32((1.0/(user_settings.max_fps as f32) -  ui.io().delta_time).max(0.0)));

    }, if fullscreen {Some(Fullscreen::Borderless(None))} else {None},  &mut ctx);

}


pub fn relaunch_windows(admin: bool) {
    
    let command = if admin {
        format!("Start-Process \"{}\" -Verb RunAs", current_exe().unwrap().as_os_str().to_str().unwrap())
    }else {
        format!("Start-Process \"{}\"", current_exe().unwrap().as_os_str().to_str().unwrap())
    };
    println!("{command}");

    // win_msgbox::show::<Okay>(&format!("command: {}", command));

    std::process::Command::new("powershell")
    .arg(command)
    .spawn().expect("cannot spawn command");
    exit(0);
}