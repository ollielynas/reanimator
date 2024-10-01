use std::{
    env::{self, current_exe}, fs, os::windows::process::CommandExt, path::PathBuf, process::{exit, Command}, ptr, str::FromStr, thread::{self, Thread}
};

use enum_to_string::ToJsonString;
use humantime::Duration;
use imgui::{sys, Ui};
use imgui_winit_support::winit::window::Fullscreen;
use itertools::Itertools;
use log::{error, info, log};
use platform_dirs::AppDirs;
use rfd::FileDialog;
use self_update::cargo_crate_version;
use serde::Serialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use win_msgbox::Okay;
use winapi::um::winbase::CREATE_NO_WINDOW;

use crate::{
    relaunch_program,
    support::{create_context, init_with_startup},
    user_info::{UserSettings, USER_SETTINGS_SAVEFILE_VERSION},
    widgets::{link_widget, path_input},
};

#[derive(EnumIter, ToJsonString, Serialize)]
enum SetupStage {
    ProjectFolder,
    InstallFfmpeg,
    DefaultApplicationForRepjFiles,
}

/// call this right at the start
pub fn set_panic_hook() {

    let args: Vec<String> = env::args().collect();

    if args.len() == 3 && args[1] == "-ErrorMessage" {
    
    
    
    let message =
        String::from_utf8(fast_smaz::decompress(&hex::decode(args[2].clone()).unwrap_or_default()).unwrap_or_default())
        .unwrap_or_default();


    let mut ctx: imgui::Context = create_context();
    let app_dirs = match AppDirs::new(Some("ReAnimator"), false) {
        Some(a) => a.config_dir,
        None => current_exe().unwrap(),
    };
    

    let mut user_settings: UserSettings = savefile::load_file(
        app_dirs.join("settings.bat"),
        USER_SETTINGS_SAVEFILE_VERSION,
    )
    .unwrap_or_default();

    user_settings.load_theme(&mut ctx);

    init_with_startup(
        "Error!",
        Some((512,356)),
        |_, _, _display| {},
        move |_, ui, _display, _rendererr, _drop_filee, _window| {
            
            let size_array = ui.io().display_size;
            let _a: imgui::StyleStackToken<'_> = ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0));
            ui.window("setup")
                .always_auto_resize(true)
                .menu_bar(false)
                .bg_alpha(0.0)
                .focused(true)
                .no_decoration()
                .title_bar(false)
                .size_constraints(size_array, size_array)
                .position([0.0,0.0], imgui::Condition::Always)
                .build(|| {
                    ui.set_window_font_scale(1.2);
                ui.text("The program has crashed");
                ui.set_window_font_scale(1.0);
                
                ui.input_text_multiline("##", &mut message.clone(), [
                    (ui.calc_text_size(&message)[0] + ui.clone_style().frame_padding[1] * 2.0).min(ui.content_region_avail()[0]),
                    ui.calc_text_size(&message)[1] + ui.clone_style().frame_padding[1] * 2.0
                    ]).build();
                if ui.clipboard_text().as_ref() != Some(&message) {
                if ui.button("copy error message") {
                    ui.set_clipboard_text(&message);
                }}else {
                    ui.text("copied message");
                }
                if ui.button("submit bug report") {
                    let bug_body = "Try to include info like:%0A- how to replicate the issue%0A- what the issue was%0A-screenshots%0A- error messages%0A- an exported version of the project%A0%0Adon't forget a title".replace(" ", "%20");
                    open::that(format!("https://github.com/ollielynas/reanimator/issues/new?labels=bug&title=[bug]%20v{}-&body=\"{}\"", cargo_crate_version!(), bug_body));
                }

                if ui.button("relaunch") {
                    relaunch_program(false, "");
                }
                
                });

        },
        if false {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        },
        &mut ctx,
    );
        exit(0);
    }

    std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo<'_>| {

        let text = match (
            info.payload().downcast_ref::<&str>(),
            info.payload().downcast_ref::<String>(),
        )
        {
            (Some(a), _) => a.to_string(),
            (_, Some(a)) => a.to_owned(),
            _ => format!("{info:#?}"),
        };
    
    let message = format!(
        "version: {}\n{}message:{}",
        cargo_crate_version!(),
        if info.location().is_some() {format!("location: {} {} {}\n", info.location().unwrap().file(), info.location().unwrap().line(), info.location().unwrap().column())} else {String::new()},
        text,
    );
    
    relaunch_program(false, format!("-ErrorMessage {}",hex::encode(fast_smaz::encode(&message))));

}));
}

pub fn setup_popup(settings: &UserSettings) {
    let mut settings = settings.clone();
    let mut finished = false;
    let mut pick_options = true;
    let mut repj = true;

    let mut ffmpeg_thread: Vec<std::thread::JoinHandle<()>> = vec![];

    let mut ctx: imgui::Context = create_context();

    settings.load_theme(&mut ctx);

    let setup_items: Vec<SetupStage> = SetupStage::iter().collect();
    let mut setup_index = 0;
    init_with_startup(
        "Setup",
        Some((355,355)),
        |_, _, _display| {},
        move |_, ui, _display, renderer, drop_file, window| {
            let size_array = ui.io().display_size;
            let a: imgui::StyleStackToken<'_> = ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0));
            ui.window("setup")
                // .always_auto_resize(true)
                .menu_bar(false)
                .bg_alpha(0.0)                
                .no_decoration()
                .title_bar(false)
                .size(size_array, imgui::Condition::Always)
                .bring_to_front_on_focus(false)
                .position(
                    [0.0,0.0],
                    imgui::Condition::Always,
                )
                // .size_constraints([-1.0,-1.0], [size_array[0], size_array[1]])
                // .position_pivot([0.5, 0.5])
                .build(|| {
                    if pick_options {
                        ui.set_window_font_scale(1.2);
                        ui.text("Setup");
                        ui.set_window_font_scale(1.0);
                        ui.text_disabled(format!("{}, {}/{}", setup_items[setup_index], setup_index + 1, setup_items.len()));

                        ui.spacing();
                        ui.spacing();

                        match setup_items[setup_index] {
                            SetupStage::ProjectFolder => {
                                ui.text("set project folder");
                                ui.set_next_item_width(-10.0);
                                path_input(ui, "##", &mut settings.project_folder_path);
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
                            }
                            SetupStage::InstallFfmpeg => {
                                ui.checkbox("enable ffmpeg", &mut settings.install_ffmpeg);
                            }
                            SetupStage::DefaultApplicationForRepjFiles => {
                                ui.checkbox("set as default for .repj files", &mut repj);
                            }
                        }
                        
                    } else {
                        if setup_index >= setup_items.len() {
                            finished = true;
                            settings.finished_setup = true;
                            settings.save();
                            relaunch_program(false, "");
                        }

                        match setup_items[setup_index] {
                            SetupStage::ProjectFolder => {
                                ui.text("setting up project folder");
                                setup_index += 1;
                            }
                            SetupStage::InstallFfmpeg => {
                                ui.text(format!("{}, installing/updating ffmpeg", spinner(ui)));
                                match ffmpeg_thread.len() {
                                    0 => {
                                        let jh = thread::spawn(move || {
                                            if settings.install_ffmpeg {
                                                info!(
                                                    "downloaded ffmpeg: {:?}",
                                                    ffmpeg_sidecar::download::auto_download()
                                                        .unwrap()
                                                );
                                            };
                                        });
                                        ffmpeg_thread.push(jh);
                                    }
                                    _ => {
                                        if ffmpeg_thread[0].is_finished() {
                                            setup_index += 1;
                                        }
                                    }
                                }
                            }
                            SetupStage::DefaultApplicationForRepjFiles => {
                                if repj {
                                    repj = false;
                                    set_as_default_for_filetype2();
                                    setup_index += 1;
                                }
                            }
                        }
                    }
                });
                ui.window("back-fwd")
                .always_auto_resize(true)
                .menu_bar(false)
                .bg_alpha(0.0)
                // .focused(true)
                .bring_to_front_on_focus(true)
                .no_decoration()
                .title_bar(false)
                .position(
                    size_array,
                    imgui::Condition::Always,
                )
                .size_constraints([-1.0,-1.0], [size_array[0], size_array[1]])
                .position_pivot([1.0, 1.0])
                .build(|| {
                    ui.disabled(setup_index <= 0, || {
                        if ui.button("back") {
                            setup_index -= 1;
                        };
                    });
                    ui.same_line();
                    if setup_index < setup_items.len() - 1 {
                        if ui.button("next") {
                            setup_index += 1;
                            println!("{setup_index}")
                        };
                    } else {
                        if ui.button("finish") {
                            pick_options = false;
                            setup_index = 0;
                        };
                    }
                });
        },
        if false {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        },
        &mut ctx,
    );

    exit(0);
}

/// TODO add support for linux
pub fn set_as_default_for_filetype() {
    let command = format!(
        "ftype \"ReAnimator Project\"=\"{}\"  \"%1\" && assoc .repj=\"ReAnimator Project\"",
        current_exe().unwrap().as_os_str().to_str().unwrap()
    );
    let path = current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("ReAnimator Setup Util.bat");
    fs::write(
        current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("ReAnimator Setup Util.bat"),
        command,
    );
    // command = command.replace("\"", "\\`\"");
    let res = Command::new("powershell")
        .arg(format!(
            "Start-Process -FilePath \"{}\" -Verb RunAs",
            path.display().to_string()
        ))
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    println!(
        "Start-Process -FilePath \"{}\" -Verb RunAs",
        path.display().to_string()
    );
}

pub fn spinner(ui: &Ui) -> String {
    let s = "⡀⡁⡂⡃⡄⡅⡆⡇⡈⡉⡊⡋⡌⡍⡎⡏⡐⡑⡒⡓⡔⡕⡖⡗⡘⡙⡚⡛⡜⡝⡞⡟⡠⡡⡢⡣⡤⡥⡦⡧⡨⡩⡪⡫⡬⡭⡮⡯⡰⡱⡲⡳⡴⡵⡶⡷⡸⡹⡺⡻⡼⡽⡾⡿⢀⢁⢂⢃⢄⢅⢆⢇⢈⢉⢊⢋⢌⢍⢎⢏⢐⢑⢒⢓⢔⢕⢖⢗⢘⢙⢚⢛⢜⢝⢞⢟⢠⢡⢢⢣⢤⢥⢦⢧⢨⢩⢪⢫⢬⢭⢮⢯⢰⢱⢲⢳⢴⢵⢶⢷⢸⢹⢺⢻⢼⢽⢾⢿⣀⣁⣂⣃⣄⣅⣆⣇⣈⣉⣊⣋⣌⣍⣎⣏⣐⣑⣒⣓⣔⣕⣖⣗⣘⣙⣚⣛⣜⣝⣞⣟⣠⣡⣢⣣⣤⣥⣦⣧⣨⣩⣪⣫⣬⣭⣮⣯⣰⣱⣲⣳⣴⣵⣶⣷⣸⣹⣺⣻⣼⣽⣾⣿";
    let index = ((ui.time() * 3.0) % s.chars().count() as f64).floor() as usize;
    return s
        .get(index..index)
        .unwrap_or(if (ui.time() as i32) % 2 == 0 {
            "|"
        } else {
            "-"
        })
        .to_owned();
}

pub fn set_as_default_for_filetype2() {
    let command = format!(
        "ftype \"ReAnimator Project\"=\"{}\"  \"%1\" && assoc .repj=\"ReAnimator Project\"",
        current_exe().unwrap().as_os_str().to_str().unwrap()
    );
    info!("{command}");
    let res = Command::new("cmd")
        .raw_arg("/C ".to_owned() + &command)
        .creation_flags(CREATE_NO_WINDOW)
        .output();
}


pub fn update() -> Result<(), Box<dyn (::std::error::Error)>> {
    info!("updating");

    let relase_builds = self_update::backends::github::ReleaseList::configure()
        .repo_owner("ollielynas")
        .repo_name("reanimator")
        .build();

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

            set_as_default_for_filetype2();
            relaunch_program(false, "");
        
        exit(0);
    }
    info!("Update status: `{}`!", status.version());

    Ok(())
}