use std::{env::current_exe, fs, os::windows::process::CommandExt, process::Command, thread::{self, Thread}};

use enum_to_string::ToJsonString;
use humantime::Duration;
use imgui::Ui;
use imgui_winit_support::winit::window::Fullscreen;
use itertools::Itertools;
use log::{info, log};
use rfd::FileDialog;
use serde::Serialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use win_msgbox::Okay;
use winapi::um::winbase::CREATE_NO_WINDOW;

use crate::{relaunch_windows, support::{create_context, init_with_startup}, user_info::UserSettings, widgets::link_widget};


#[derive(EnumIter, ToJsonString, Serialize)]
enum SetupStage {
    ProjectFolder,
    InstallFfmpeg,
    DefaultApplicationForRepjFiles,
}



pub fn setup_popup(settings: &UserSettings) -> bool {
    let mut settings = settings.clone();
    let mut finished = false;
    let mut pick_options = true;
    let mut repj = true;

    let mut ffmpeg_thread: Vec<std::thread::JoinHandle<()>> = vec![];

    let mut ctx: imgui::Context = create_context();

    settings.load_theme(&mut ctx);

    let setup_items: Vec<SetupStage> = SetupStage::iter().collect();
    let mut setup_index = 0;

    init_with_startup("ReAnimator", |_, _, _display| {
    }, move |_, ui, _display, renderer, drop_file, window| {
        
        let size_array = ui.io().display_size;

    ui.window("setup")
    .always_auto_resize(true)
    .menu_bar(false)
    .bg_alpha(0.0)
    .focused(true)
    .no_decoration()
    .title_bar(false)
    .position([size_array[0] * 0.5, size_array[1] * 0.5], imgui::Condition::Always)
    .position_pivot([0.5,0.5]).build(|| {

        if pick_options {
        // ui.set_window_font_scale(1.5);
        // ui.text("Setup");
        // ui.set_window_font_scale(1.0);
        // ui.text_disabled(format!("{}, {}/{}", setup_items[setup_index], setup_index + 1, setup_items.len()));

        // ui.spacing();

        match setup_items[setup_index] {
            SetupStage::ProjectFolder => {
                ui.text("project folder");
                ui.text(settings.project_folder_path.display().to_string());
                if ui.button("change path") {
                    let new_folder = FileDialog::new()
                            .set_directory(&settings.project_folder_path)
                            .set_can_create_directories(true)
                            .set_title("set project folder")
                            .pick_folder();
                        if new_folder.is_some() {
                            settings.project_folder_path = new_folder.unwrap();
                            settings.save();
                        }
                }
            },
            SetupStage::InstallFfmpeg => {
                ui.checkbox("enable ffmpeg", &mut settings.install_ffmpeg);
            },
            SetupStage::DefaultApplicationForRepjFiles => {
                ui.checkbox("set as default for .repj files", &mut repj);
            },
        }
        ui.disabled(setup_index <= 0, || {
            if ui.button("back") {
                setup_index -= 1;
            };
        });
        ui.same_line();
        if setup_index < setup_items.len() -1 {
        if ui.button("next") {
            setup_index += 1;
        };
    }else {
            if ui.button("finish") {
                pick_options = false;
                setup_index = 0;
            };

        }
    } else {
        if setup_index >= setup_items.len() {
            finished = true;
            settings.save();
            relaunch_windows(false);
        }

        match setup_items[setup_index] {
            SetupStage::ProjectFolder => {
                ui.text("setting up project folder");
                setup_index += 1;
            },
            SetupStage::InstallFfmpeg => {
                ui.text(format!("{}, installing/updating ffmpeg", spinner(ui)));
                match ffmpeg_thread.len() {
                    0 => {
                        let jh = thread::spawn(move || {
                            if settings.install_ffmpeg {
                                info!("downloaded ffmpeg: {:?}", ffmpeg_sidecar::download::auto_download().unwrap());
                            };
                        });
                        ffmpeg_thread.push(jh);
                    },
                    _ => {
                        if ffmpeg_thread[0].is_finished() {
                            setup_index += 1;
                        }
                    },
                }
            },
            SetupStage::DefaultApplicationForRepjFiles => {
                if repj {
                    repj = false;
                    set_as_default_for_filetype2();
                    setup_index += 1;
                }
            },
        }
    
    }
        
    });


    }, if false {Some(Fullscreen::Borderless(None))} else {None},  &mut ctx);


    return finished;
}

/// TODO add support for linux
pub fn set_as_default_for_filetype() {
    let command = format!("ftype \"ReAnimator Project\"=\"{}\"  \"%1\" && assoc .repj=\"ReAnimator Project\"", current_exe().unwrap().as_os_str().to_str().unwrap());
    let path = current_exe().unwrap().parent().unwrap().join("ReAnimator Setup Util.bat");
    fs::write(current_exe().unwrap().parent().unwrap().join("ReAnimator Setup Util.bat"), command);
    // command = command.replace("\"", "\\`\"");
    let res = Command::new("powershell")
    .arg(format!("Start-Process -FilePath \"{}\" -Verb RunAs", path.display().to_string()))
    .creation_flags(CREATE_NO_WINDOW)
    .output();
println!("Start-Process -FilePath \"{}\" -Verb RunAs", path.display().to_string());
}


pub fn spinner(ui:&Ui) -> String {
    let s = "⡀⡁⡂⡃⡄⡅⡆⡇⡈⡉⡊⡋⡌⡍⡎⡏⡐⡑⡒⡓⡔⡕⡖⡗⡘⡙⡚⡛⡜⡝⡞⡟⡠⡡⡢⡣⡤⡥⡦⡧⡨⡩⡪⡫⡬⡭⡮⡯⡰⡱⡲⡳⡴⡵⡶⡷⡸⡹⡺⡻⡼⡽⡾⡿⢀⢁⢂⢃⢄⢅⢆⢇⢈⢉⢊⢋⢌⢍⢎⢏⢐⢑⢒⢓⢔⢕⢖⢗⢘⢙⢚⢛⢜⢝⢞⢟⢠⢡⢢⢣⢤⢥⢦⢧⢨⢩⢪⢫⢬⢭⢮⢯⢰⢱⢲⢳⢴⢵⢶⢷⢸⢹⢺⢻⢼⢽⢾⢿⣀⣁⣂⣃⣄⣅⣆⣇⣈⣉⣊⣋⣌⣍⣎⣏⣐⣑⣒⣓⣔⣕⣖⣗⣘⣙⣚⣛⣜⣝⣞⣟⣠⣡⣢⣣⣤⣥⣦⣧⣨⣩⣪⣫⣬⣭⣮⣯⣰⣱⣲⣳⣴⣵⣶⣷⣸⣹⣺⣻⣼⣽⣾⣿";
    let index = ((ui.time() * 3.0) % s.chars().count() as f64).floor() as usize;
    return s.get(index..index).unwrap_or(if (ui.time() as i32) % 2 == 0 { "|" } else {"-"}).to_owned()
}

pub fn set_as_default_for_filetype2() {

    let command = format!("ftype \"ReAnimator Project\"=\"{}\"  \"%1\" && assoc .repj=\"ReAnimator Project\"", current_exe().unwrap().as_os_str().to_str().unwrap());
    info!("{command}");
    let res = Command::new("cmd")
    .raw_arg("/C ".to_owned() + &command)
    .creation_flags(CREATE_NO_WINDOW)
    .output();

}