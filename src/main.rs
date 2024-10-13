#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use std::{
    env::{self, current_exe}, os::windows::process::CommandExt, path::Path, process::{exit, Command}, sync::Mutex, thread::sleep, time::{Duration, Instant}
};

use debug_and_logger::set_logger_mine;
use fonts::MyFonts;
use imgui_winit_support::winit::{
    dpi::{LogicalSize},
    window::{Fullscreen},
};
use winreg::enums::HKEY_CURRENT_USER;
use import_export::load_project;
use lazy_static::lazy_static;
use log::{info};
use platform_dirs::{AppDirs};
use popups::{set_panic_hook, setup_popup, update};
use project::Project;
use savefile;
use self_update::cargo_crate_version;
use winreg::RegKey;

use support::{create_context, init_with_startup};
use system_extensions::dialogues::messagebox::{IconType, MessageBox, WindowType};
use user_info::{UserSettings, USER_SETTINGS_SAVEFILE_VERSION};

use winapi::um::winbase::CREATE_NO_WINDOW;

use perf_monitor::cpu::{ThreadStat, ProcessStat, processor_numbers};


lazy_static! {
    pub static ref LOG_TEXT: Mutex<Vec<Vec<u8>>> = Mutex::new(vec![]);
}

#[macro_use]
extern crate savefile_derive;

pub mod advanced_color_picker;
pub mod batch_edit;
pub mod debug_and_logger;
pub mod fonts;
pub mod generic_io;
pub mod generic_node_info;
pub mod history_tracker;
pub mod import_export;
pub mod node;
pub mod nodes;
pub mod popups;
pub mod project;
pub mod project_files;
pub mod project_settings;
pub mod render_nodes;
pub mod sidebar;
pub mod storage;
pub mod support;
pub mod user_info;
pub mod widgets;

fn main() -> anyhow::Result<()> {

    // #[cfg(debug_assertions)] {
        set_logger_mine()?;
    // }
    
    set_panic_hook();


    let a = update();

    // move this to popups.rs
    if let Err(a2) = a {
        if a2.to_string().contains("os error 5") {
            if MessageBox::new("Updater", "New update available, press ok to install")
                .set_icon_type(IconType::ICON_INFORMATION)
                .set_window_type(WindowType::OK_CANCEL)
                .show()
                .is_ok()
            {
                relaunch_program(true, "");
            };
        } else {
            
        }
    }

    let args: Vec<String> = env::args().collect();

    info!("args {args:?}");

    let res = Command::new("cmd")
        .raw_arg("/C ".to_owned() + "assoc .repj")
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let assoc = match res {
        Ok(a) => a.status.success(),
        Err(_) => false,
    };

    // update();

    let app_dirs = match AppDirs::new(Some("ReAnimator"), false) {
        Some(a) => a.config_dir,
        None => current_exe().unwrap(),
    };

    let fonts = MyFonts::new();

    let mut user_settings: UserSettings = savefile::load_file(
        app_dirs.join("settings.bat"),
        USER_SETTINGS_SAVEFILE_VERSION,
    )
    .unwrap_or_default();

    if !user_settings.finished_setup {
        setup_popup(&user_settings);
    }

    let mut project: Option<Project> = None;

    if !assoc {
        if MessageBox::new("ReAnimator", "ReAnimator has not been set a the default application for .repj files. Press Ok to set as default")
                .set_icon_type(IconType::ICON_WARNING)
                .set_window_type(WindowType::OK_CANCEL)
                .show().is_ok() {
                    relaunch_program(true, "");
                };
    }

    if !user_settings.finished_setup && false {
        setup_popup(&user_settings);
    }

    user_settings.update_projects();

    let mut return_to_home = false;

    let mut ctx: imgui::Context = create_context();

    

    let mut save_timer = Instant::now();

    let mut settings_window_open = false;

    user_settings.load_theme(&mut ctx);

    let fullscreen = user_settings.fullscreen;

    let mut loaded_project = false;
    let visible = true;

    init_with_startup(
        "ReAnimator",
        None,
        visible,
        |_, _, _display| {
            
        },
        move |_, ui, display, renderer, drop_file, window| {

            if !loaded_project && args.len() >= 2 && args[1].contains(".repj") {
                if let Some(p) = load_project(&args[1], &user_settings) {
                    project = Some(Project::new(&p, display.clone()));
                    loaded_project = true;
                }
            }
            let mut stat_p: Result<ProcessStat, std::io::Error> = Err(std::io::Error::other("not debug mode"));
            let mut stat_t: Result<ThreadStat, std::io::Error> =   Err(std::io::Error::other("not debug mode"));
            #[cfg(debug_assertions)] {
            let _core_num = processor_numbers().unwrap();
            stat_p = ProcessStat::cur();
            stat_t = ThreadStat::cur();
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
                let _ = window.request_inner_size(LogicalSize::new(1024, 512));
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
                            Ok(_) => {}
                            Err(e) => {
                                info!("{e}")
                            }
                        }
                    } else {
                        if window.is_maximized() {
                            project.project_settings.maximised = true;
                        } else {
                            if let Ok(pos) = window.outer_position() {
                                project.project_settings.window_pos =
                                    Some([pos.x as f32, pos.y as f32]);
                            }
                            let size = window.inner_size();
                            project.project_settings.window_size =
                                Some([size.width as f32, size.height as f32]);
                        }

                        let _ = project.save();
                    }
                }
            } else {
                project = Project::project_menu(ui, display, &mut user_settings, renderer);
                ui.window("settings button")
                    .draw_background(false)
                    .movable(false)
                    .no_decoration()
                    .position_pivot([0.0, 1.0])
                    .size_constraints(
                        [ui.calc_text_size("settings xxxxxxxx")[0], -1.0],
                        [9999.0, -1.0],
                    )
                    .position([10.0, size_array[1] + 5.0], imgui::Condition::Always)
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
                    .position_pivot([0.5, 1.0])
                    .size_constraints([ui.calc_text_size("")[0], -1.0], [9999.0, -1.0])
                    .position(
                        [size_array[0] * 0.5, size_array[1]],
                        imgui::Condition::Always,
                    )
                    .build(|| {
                        ui.text("v".to_owned() + cargo_crate_version!());
                        // cargo_crate_version!()
                    });
            }

            if settings_window_open {
                user_settings.settings_window(ui, &mut settings_window_open, &fonts);
            }
            #[cfg(debug_assertions)] {
            if ui.io().key_alt {
                let usage_p = stat_p.unwrap().cpu().unwrap() * 100f64;
                let usage_t = stat_t.unwrap().cpu().unwrap() * 100f64;


                debug_and_logger::profile(ui, usage_p, usage_t);
            }}
            
            sleep(Duration::from_secs_f32(
                (1.0 / (user_settings.max_fps as f32)
                    - (Instant::now() - frame_start).as_secs_f32())
                .max(0.0),
            ));



        },
        if fullscreen {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        },
        &mut ctx,
    );

    ctx.suspend();

    return Ok(());
}

/// TODO: make this cross platform
pub fn relaunch_program(admin: bool, args: impl Into<String>) {
    let command = if admin {
        format!(
            "Start-Process \"{}\" -ArgumentList \"{}\" -Verb RunAs",
            current_exe().unwrap().as_os_str().to_str().unwrap(),
            args.into()
        )
    } else {
        format!(
            "Start-Process \"{}\" -ArgumentList \"{}\"",
            current_exe().unwrap().as_os_str().to_str().unwrap(),
            args.into()
        )
    }.replace("-ArgumentList \"\"", "");
    info!("{command}");

    // win_msgbox::show::<Okay>(&format!("command: {}", command));

    std::process::Command::new("powershell")
        .arg(command)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .expect("cannot spawn command");
    exit(0);
}


pub fn set_gpu_pref() -> anyhow::Result<()> {


    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("Software")
        .join("Microsoft")
        .join("DirectX")
        .join("UserGpuPreferences")
        ;
    let (key, disp) = hkcu.create_subkey(&path)?;
        match disp {
            winreg::enums::RegDisposition::REG_CREATED_NEW_KEY => info!("A new key has been created"),
            winreg::enums::RegDisposition::REG_OPENED_EXISTING_KEY => info!("An existing key has been opened"),
        }
        
        key.set_value(current_exe()?.display().to_string(), &"GpuPreference=2;")?;
        // let value = key.get_value::<String, _>(current_exe()?.display());
        // println!("{:?} {:?}", current_exe()?.display().to_string(), value);
    return Ok(());
}