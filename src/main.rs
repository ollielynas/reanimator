#![cfg_attr(
    all(
      target_os = "windows",
      not(debug_assertions),
    ),
    windows_subsystem = "windows"
  )]

use std::{borrow::BorrowMut, env::{self, current_exe}, fs, ops::DerefMut, os::windows::process::CommandExt, process::{exit, Command}, sync::Mutex, thread::{sleep, sleep_ms}, time::{Duration, Instant, SystemTime}};


use fonts::MyFonts;
use imgui_winit_support::winit::{dpi::{LogicalPosition, LogicalSize}, error::OsError, monitor::VideoMode, window::{self, Fullscreen, Window, WindowBuilder}};
use import_export::load_project;
use platform_dirs::{AppDirs, UserDirs};
use savefile;
use project::Project;
use self_update::cargo_crate_version;
use support::{create_context, init_with_startup};
use system_extensions::dialogues::messagebox::{IconType, MessageBox, WindowType};
use user_info::{UserSettings, USER_SETTINGS_SAVEFILE_VERSION};
use win_msgbox::{raw::w, Okay};
use winapi::um::winbase::CREATE_NO_WINDOW;
use log::{info, trace, error};
use std::sync::RwLock;
use lazy_static::lazy_static;
use fast_smaz::Smaz;

lazy_static! {
    pub static ref LOG_TEXT: Mutex<Vec<Vec<u8>>> = Mutex::new(vec![]);
}




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
pub mod project_settings;
pub mod generic_io;
pub mod sidebar;
pub mod batch_edit;
pub mod import_export;
pub mod generic_node_info;
pub mod project_files;


// in theoiry this is just a temp solution, but im never going to 
// static DISPLAY_TEXTURE_ID: Mutex<Option<TextureId>> = Mutex::new(None);


pub fn set_as_default_for_filetype(popup_message: bool) {

    let command = format!("ftype \"ReAnimator Project\"=\"{}\"  \"%1\" && assoc .repj=\"ReAnimator Project\"", current_exe().unwrap().as_os_str().to_str().unwrap());
    info!("{command}");
    let res = Command::new("cmd")
    .raw_arg("/C ".to_owned() + &command)
    .creation_flags(CREATE_NO_WINDOW)
    .output();
    if popup_message {
        win_msgbox::show::<Okay>(&format!("{} \n\n\n {:?}", command, res));
    }
}


fn update() -> Result<(), Box<dyn (::std::error::Error)>> {
    info!("updating");

    
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

            set_as_default_for_filetype(false);

            win_msgbox::show::<Okay>(&format!("Updated to version: {}", status.version()));
            relaunch_windows(false);
        }
        exit(0);
    }
    // self_update
    info!("Update status: `{}`!", status.version());
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // env::set_var("MY_LOG_STYLE", "info");
    // env_logger::init();

    // todo: achuallly do something with this
    // color_eyre::install().unwrap();

    // let shared_data = RwLock::new(Vec::<i32>::new());

    let shared_dispatch = fern::Dispatch::new().into_shared();

    
    fern::Dispatch::new()
    // Perform allocation-free log formatting
    .format(|out, message, record| {
        
        if let Ok(ref mut log) = LOG_TEXT.lock() {
            let text = format!(
                "\n[{} {}:{}:0 {}] {}",
                record.level(),
                record.file().unwrap_or_default(),
                record.line().unwrap_or(999999).to_string().replace("999999", ""),
                humantime::format_rfc3339(SystemTime::now()),
                message
            );
            match log.last_mut() {
                Some(a) if a.len() < 500 => {
                    *a = [a.smaz_decompress().unwrap_or_default(), text.as_bytes().to_vec()].concat().smaz_compress();
                },
                _ => {
                    log.push(text.smaz_compress())
                },
            }
        }

        out.finish(format_args!(
            "[{} {}:{}:0 {}] {}",
            record.level(),
            record.file().unwrap_or_default(),
            record.line().unwrap_or(999999).to_string().replace("999999", ""),
            humantime::format_rfc3339(SystemTime::now()),
            message
        ))
    })
    // Add blanket level filter -
    .level(log::LevelFilter::Debug)
    // - and per-module overrides
    
    // Output to stdout, files, and other Dispatch configurations
    .chain(std::io::stdout())
    .chain(fern::log_file("output.log")?)
    // Apply globally
    .chain(shared_dispatch.clone())
    .apply()?;


    // env_logger::

    #[cfg(all(target_os="windows", not(debug_assertions)))]{
    std::panic::set_hook(Box::new(|a| {
        win_msgbox::show::<Okay>(&format!("Program Crashed \n {a}"));
    }));
        let a = update();
        fs::remove_file("output.log");

        if let Err(a2) = a {

            if a2.to_string().contains("os error 5") {
                if MessageBox::new("Updater", "New update avalible, press ok to install")
                .set_icon_type(IconType::ICON_INFORMATION)
                .set_window_type(WindowType::OK_CANCEL)
                .show().is_ok() {
                    relaunch_windows(true);
                };
            }else {
                win_msgbox::show::<Okay>(&format!("Error Updating \n {a2}"));
            }
            // panic!("{:?}", a2);
        }
    }
    // panic!("test");



    let args: Vec<String> = env::args().collect();

    trace!("{args:?}");

    set_as_default_for_filetype(false);


    let res = Command::new("cmd")
    .raw_arg("/C ".to_owned()+ "assoc .repj")
    .creation_flags(CREATE_NO_WINDOW)
    .output();

    

    let assoc = match res {
        Ok(a) => a.status.success(),
        Err(_) => false,
    };


    if !assoc {
        if MessageBox::new("ReAnimator", "ReAnimator has not been set a the default application for .repj files. Press Ok to set as default")
                .set_icon_type(IconType::ICON_WARNING)
                .set_window_type(WindowType::OK_CANCEL)
                .show().is_ok() {
                    relaunch_windows(true);
                };
    }



    // update();

    let app_dirs = match AppDirs::new(Some("Reanimator"), false) {
        Some(a) => {
            a.config_dir
        },
        None => {
            current_exe().unwrap()
        },
    };

    let fonts = MyFonts::new();


    let mut user_settings: UserSettings = savefile::load_file(app_dirs.join("settings.bat"), USER_SETTINGS_SAVEFILE_VERSION).unwrap_or_default();
    let mut project: Option<Project> = None;
    
    
    user_settings.update_projects();



    let mut return_to_home = false;
    
    let mut ctx: imgui::Context = create_context();

    
    let mut save_timer  = Instant::now();

    let mut settings_window_open = false;
    
    

    user_settings.load_theme(&mut ctx);

    let fullscreen = user_settings.fullscreen; 

    let mut loaded_project = false;


    init_with_startup("ReAnimator", |_, _, _display| {
    }, move |_, ui, display, renderer, drop_file, window| {


        if args.len() >= 2 && args[1].contains(".repj") && !loaded_project {
            if let Some(p) = load_project(args[1].clone(), user_settings.clone()) {
                project = Some(Project::new(p, display.clone()));
                loaded_project = true;
            }
        }

        let frame_start = Instant::now();
        let mut global_font_tokens = vec![];
        if let Some(font_id) = user_settings.font_id {
        if ui.fonts().get_font(font_id).is_some() {
            global_font_tokens.push(ui.push_font(font_id));
        }
        }


        if return_to_home {
            project = None;
            return_to_home = false;
            window.set_maximized(false);
            window.request_inner_size(LogicalSize::new(1024, 512));

        }

        let size_array = ui.io().display_size;



        if let Some(ref mut project) = project {


            if let Some(path) = drop_file {
                project.drop_file(path, ui);
            } 
            project.render(ui, &mut user_settings, renderer, window);
            return_to_home = project.return_to_home_menu;
            // ui.show_default_style_editor();
            if save_timer.elapsed().as_secs_f32() > 2.0 {
                save_timer = Instant::now();
                if user_settings.history {
                let r = project.update_history_and_save();
                match r {
                    Ok(_) => {},
                    Err(e) => {
                        info!("{e}")},
                }
                }else {
                    
                    if window.is_maximized() {
                        project.project_settings.maximised = true;
                    }else {
                        if let Ok(pos) = window.outer_position() {
                            project.project_settings.window_pos = Some([pos.x as f32, pos.y as f32]); 
                        }
                        let size = window.inner_size();
                        project.project_settings.window_size = Some([size.width as f32, size.height as f32]); 
                    }

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
            ui.window("v")
        .draw_background(false)
        .movable(false)
        .no_decoration()
        .position_pivot([0.5,1.0])
        .size_constraints([ui.calc_text_size("")[0],-1.0], [9999.0,-1.0])
        .position([size_array[0]*0.5,size_array[1]], imgui::Condition::Always)
        .build(|| {
            ui.text("v".to_owned()+cargo_crate_version!());
            // cargo_crate_version!()
        });
        }

        if settings_window_open {
            user_settings.settings_window(ui, &mut settings_window_open, &fonts);
        }
        sleep( Duration::from_secs_f32((1.0/(user_settings.max_fps as f32) - (Instant::now() - frame_start).as_secs_f32()).max(0.0)));

    }, if fullscreen {Some(Fullscreen::Borderless(None))} else {None},  &mut ctx);

    return Ok(());
}


pub fn relaunch_windows(admin: bool) {
    
    let command = if admin {
        format!("Start-Process \"{}\" -Verb RunAs", current_exe().unwrap().as_os_str().to_str().unwrap())
    }else {
        format!("Start-Process \"{}\"", current_exe().unwrap().as_os_str().to_str().unwrap())
    };
    info!("{command}");

    // win_msgbox::show::<Okay>(&format!("command: {}", command));

    std::process::Command::new("powershell")
    .arg(command)
    .creation_flags(CREATE_NO_WINDOW)
    .spawn()
    
    .expect("cannot spawn command");
    exit(0);
}