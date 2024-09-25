#![allow(warnings)]

use glium::debug::TimestampQuery;
use glium::texture::RawImage2d;
use glium::{program, BlitTarget, Display, Program, Rect, Surface};
use imgui::drag_drop::PayloadIsWrongType;
use imgui::{sys::ImVec2, ImColor32, TreeNodeToken, Ui};
use imgui::{Style, WindowFlags, WindowHoveredFlags};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::winit::dpi::{Position, Size};
use itertools::Itertools;
use platform_dirs::{AppDirs, UserDirs};
use self_update::cargo_crate_version;
use textdistance::{Algorithm, Cosine, Hamming, Levenshtein};
use std::collections::HashSet;
use std::thread::{panicking, sleep};
use std::{
    env::current_exe,
    f32::{consts::PI, NAN},
    ffi::OsString,
    hash::{DefaultHasher, Hash, Hasher},
    task::ready,
    time,
};

use glium::{backend::Facade, glutin::surface::WindowSurface, Texture2d};
use savefile::SavefileError;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use strum::IntoEnumIterator;

use crate::generic_io::EditTab;
use crate::generic_node_info::GenericNodeInfo;
use crate::node::random_id;
use crate::nodes::debug;

use crate::nodes::input::load_gif::LoadGifNode;
use crate::nodes::input::load_image::LoadImage;
use crate::nodes::output::cover_window::CoverWindowNode;
use crate::project_settings::{ProjectSettings, PROJECT_SETTINGS_VERSION};
use crate::render_nodes::RenderNodesParams;
use crate::sidebar::SidebarParams;
use crate::{
    advanced_color_picker::AdvancedColorPicker, history_tracker::Snapshot, node,
    nodes::node_enum::*,
};
use crate::{
    node::MyNode,
    storage::Storage,
    user_info::UserSettings,
};
use crate::{project, project_settings};
use rfd::FileDialog;

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

const MAX_LOADING: i32 = 5;

// #[savefile_derive]
pub struct Project {
    pub storage: Storage,
    pub nodes: Vec<Box<dyn MyNode>>,
    pub selected: Option<usize>,
    pub graph_offset: [f32; 2],
    pub scale: f32,
    pub selected_input: Option<String>,
    pub selected_output: Option<String>,
    pub connections: HashMap<String, String>,
    pub metrics: bool,
    pub new_node_types: Vec<Box<dyn MyNode>>,
    pub path: PathBuf,
    pub node_speeds: HashMap<String, Duration>,
    /// which node should have its edit window open
    pub node_edit: Option<usize>,
    pub node_run_order: (u64, Vec<String>),
    /// used by the "add node" model popup to track which node has been selected
    pub selected_node_to_add: usize,
    pub return_to_home_menu: bool,
    pub display_history: bool,
    pub snapshots: Vec<Snapshot>,
    pub selected_snapshot: i32,
    pub recenter: bool,
    pub advanced_color_picker: AdvancedColorPicker,
    pub pop_out_edit_window: HashMap<String, bool>,
    pub total_frame_time: f32,
    pub total_gpu_frame_time: f32,
    pub loading: i32,
    pub render_ticker_timer: Instant,
    pub open_settings: bool,
    pub project_settings: ProjectSettings,
    pub edit_tab: EditTab,
    pub node_search_string: String,
    pub backup_data: Vec<GenericNodeInfo>,
}

impl Project {
    // pub fn calculate_hash(&self) -> u64 {
    //     // for node in self.nodes {
    //     //     // self.storage.calculate_hash(&node.);
    //     // }
    // }

    pub fn new(path: PathBuf, display: Display<WindowSurface>) -> Project {
        // log::info!("{:?}", fs::create_dir_all(path.join("nodes")));

        let new = Project {
            advanced_color_picker: AdvancedColorPicker::default(),
            return_to_home_menu: false,
            selected_snapshot: 0,
            storage: Storage::new(display),
            nodes: vec![],
            snapshots: vec![],
            selected: None,
            graph_offset: [0.0, 0.0],
            scale: 1.0,
            selected_input: None,
            selected_output: None,
            display_history: false,
            connections: HashMap::new(),
            metrics: false,
            path: path.clone(),
            node_edit: None,
            new_node_types: vec![],
            // node_render_target: render_target(1000, 1000),
            node_speeds: HashMap::new(),
            node_run_order: (0, vec![]),
            selected_node_to_add: NodeType::iter().len(),
            recenter: true,
            pop_out_edit_window: HashMap::new(),
            total_frame_time: 0.0,
            total_gpu_frame_time: 0.0,
            loading: 0,
            project_settings: ProjectSettings::default(),
            render_ticker_timer: Instant::now(),
            open_settings: false,
            edit_tab: EditTab::BatchFileEdit,
            backup_data: vec![],
            node_search_string: String::new(),
        };
        return new;
    }

    pub fn name(&self) -> String {
        match self.path.as_path().file_name() {
            Some(a) => a.to_owned().into_string().unwrap(),
            None => format!("Name Error: {:?}", self.path),
        }
    }

    /// wont save if the project is not loaded yet
    pub fn save(&mut self) -> Result<(), SavefileError> {


        let mut input_ok = false;
        let mut output_ok = false;

        // don't save if the project hasn't finished loading
        if self.loading <= MAX_LOADING {
            return Ok(());
        }


        self.backup_data = vec![];

        // create the save folder if it doesn't already exist
        fs::create_dir_all(self.path.clone())?;
        
        // save the connections
        savefile::save_file(self.path.join("connections.bin"), 0, &self.connections)?;

        // save the settings
        savefile::save_file(
            self.path.join("project_settings.bin"),
            PROJECT_SETTINGS_VERSION,
            &self.project_settings,
        )?;

        // remove any node files that may be left over from the last save
        let _ = fs::remove_dir_all(self.path.join("nodes"));

        // create the nodes directory cos we just deleted it
        fs::create_dir_all(self.path.join("nodes"))?;
        
        // iter over nodes
        for node in &self.nodes {
            if Some(node.id()) == self.project_settings.generic_io.input_id {
                input_ok = true;
            }
            if Some(node.id()) == self.project_settings.generic_io.output_id {
                output_ok = true;
            }
            fs::create_dir_all(self.path.join("nodes").join(node.name()))?;
            self.backup_data.push(node.generic_info());
            node.save(self.path.join("nodes"))?;
        }

        // save the fact that the input id/ output id are valid
        // why am i doing this here?, I should move this
        if !input_ok {
            self.project_settings.generic_io.input_id = None;
        }
        if !output_ok {
            self.project_settings.generic_io.output_id = None;
        }

        // back up 
        savefile::save_file(self.path.join("backup_data.bin"), GenericNodeInfo::savefile_version(), &self.backup_data);

        return Ok(());
    }

    pub fn recenter_nodes(&mut self, ui: &Ui) {
        let size_array = ui.io().display_size;

        let mut average_x = 0.0;
        let mut average_y = 0.0;

        for node in &self.nodes {
            average_x += node.x();
            average_y += node.y();
        }

        average_x /= self.nodes.len() as f32;
        average_y /= self.nodes.len() as f32;

        // let center = [size_array[0] * 0.5, size_array[1] * 0.3];

        let center = screen_to_graph_pos(
            [size_array[0] * 0.5, size_array[1] * 0.3],
            self.graph_offset,
            self.scale,
        );

        for node in &mut self.nodes {
            node.set_xy(
                node.x() - average_x + center[0],
                node.y() - average_y + center[1],
            );
        }
    }

    pub fn render(
        &mut self,
        ui: &Ui,
        user_settings: &mut UserSettings,
        renderer: &mut Renderer,
        window: &imgui_winit_support::winit::window::Window,
    ) {

        // get window size
        let size_array = ui.io().display_size;
        let window_size = ImVec2::new(size_array[0], size_array[1]);


        
        // This code is responsible for loading the project in chunks so that a loading
        // screen can be sown.
        if self.loading <= MAX_LOADING + 1 {
            ui.window("loading")
                .draw_background(false)
                .no_decoration()
                .position(
                    [size_array[0] / 2.0, size_array[1] / 2.0],
                    imgui::Condition::Always,
                )
                .position_pivot([0.5, 0.5])
                .build(|| {
                    ui.set_window_font_scale(2.0);
                    ui.text(format!(
                        "{}%",
                        (
                            (self.loading as f32 / MAX_LOADING as f32) * 100.0
                            // - fastrand::f32() * (100.0 / MAX_LOADING as f32)
                        )
                        .clamp(0.0, 100.0)
                        .round()
                    ));
                    ui.set_window_font_scale(1.0);
                    log::info!("loading step: {}", self.loading);

                    if self.loading != 0 {
                        // sleep(Duration::from_secs_f32(0.25));
                    }
                    // the text id displayed before each state starts loading
                    match self.loading {
                        0 => {
                            ui.text("loading node types into memory and checking assertions");
                        }
                        1 => {
                            ui.text("Attempting to load project from memory");
                        }
                        2 => {
                            ui.text("Compiling textures and caching values");
                        }
                        3 => {
                            ui.text("Saving");
                        }
                        4 => {
                            ui.text("Verifying save integrity");
                        }
                        5 => {
                            ui.text("Updateing ffmpeg");
                        }
                        _ => {}
                    }

                    // start loading each node with a 1 tick delay
                    match self.loading - 1 {
                        -1 => {
                            // do nothing
                        }
                        0 => {
                            let mut nodes: Vec<Box<dyn MyNode>> = vec![
                                NodeType::Output.new_node(),
                                NodeType::DefaultImageOut.new_node(),
                            ];

                            let mut new_node_types: Vec<Box<dyn MyNode>> = vec![];

                            for node_type in NodeType::iter() {
                                let node: Box<dyn MyNode> = node_type.new_node();
                                debug_assert_eq!(node_type, node.type_());
                                debug_assert_eq!(node_type.name(), node.name());
                                
                                new_node_types.push(node);
                            }

                            nodes[0].set_xy(350.0, 0.0);
                            nodes[1].set_xy(0.0, 0.0);


                            self.connections.insert(nodes[0].input_id(nodes[0].inputs()[0].clone()), nodes[1].output_id(nodes[1].outputs()[0].clone()));

                            self.nodes = nodes;
                            self.new_node_types = new_node_types;

                            self.storage.project_name = self.name();

                            self.new_node_types.sort_by(|a, b| {
                                format!("{:?},{}", a.path(), a.name()).cmp(&format!(
                                    "{:?},{}",
                                    b.path(),
                                    b.name()
                                ))
                            });
                        }
                        1 => {
                            if let Ok(project_settings) =
                                savefile::load_file::<ProjectSettings, PathBuf>(
                                    self.path.join("project_settings.bin"),
                                    0,
                                )
                            {
                                self.project_settings = project_settings;
                            }
                            if let Ok(backup_data) =
                                savefile::load_file::<Vec<GenericNodeInfo>, PathBuf>(
                                    self.path.join("backup_data.bin"),
                                    GenericNodeInfo::savefile_version(),
                                )
                            {
                                self.backup_data = backup_data;
                            }
                            if let Ok(project_settings) =
                                savefile::load_file::<ProjectSettings, PathBuf>(
                                    self.path.join("project_settings.bin"),
                                    0,
                                )
                            {
                                self.project_settings = project_settings;
                            }

                            if self.project_settings.batch_files.save_path == PathBuf::new() {
                                self.project_settings.batch_files.save_path = UserDirs::new().unwrap().download_dir.join(format!("{}", self.name()));
                                fs::create_dir_all(self.project_settings.batch_files.save_path.clone());
                            }

                            if let Ok(connections) =
                                savefile::load_file::<HashMap<String, String>, PathBuf>(
                                    self.path.join("connections.bin"),
                                    0,
                                )
                            {
                                self.connections = connections;

                                self.nodes = vec![];

                                for node_type in NodeType::iter() {
                                    if let Ok(node_paths) =
                                        fs::read_dir(self.path.join("nodes").join(node_type.name()))
                                    {
                                        for node in node_paths {
                                            if let Ok(node) = node {
                                                if let Some(new_node) =
                                                    node_type.load_node(node.path())
                                                {
                                                    self.nodes.push(new_node);
                                                    log::info!("loaded node");
                                                };
                                            }
                                        }
                                    }
                                }
                                
                            } else {
                                log::info!("project not found");
                            }

                            self.storage.project_root = self.path.clone().join("root");
                            
                        }
                        2 => {
                            self.recenter_nodes(ui);

                            self.run_nodes(renderer);
                            self.run_nodes(renderer);
                            for (i, (_id, speed)) in self.node_speeds.iter().enumerate() {
                                let type_: NodeType = self.nodes[i].type_();
                                let speed = speed.as_secs_f32()/(self.total_frame_time.max(self.total_gpu_frame_time)/self.node_speeds.len() as f32);
                                if let Some(array) = user_settings.node_speed.get_mut(&self.nodes[i].name()) {
                                    array.push(speed);
                                    if array.len() > 40 {
                                        array.remove(40);
                                    }
                            }else {
                                user_settings.node_speed.insert(self.nodes[i].name(), vec![speed]);
                            }

                        }
                        #[cfg(debug_assertion)] {
                            let a = savefile::save_file("src/node_speeds.bin", 0, &user_settings.node_speed);
                            // log::info!("{a:?}");
                        
                        }
                        }
                        3 => {
                            self.save();
                            
                            if let Some(pos) = self.project_settings.window_pos {
                            if let Some(size) = self.project_settings.window_pos {
                                window.set_outer_position(Position::Physical(pos.into()));
                                let _ = window.request_inner_size(Size::Physical(size.into()));
                            }
                            }
                            if self.project_settings.maximised {
                                window.set_maximized(true);
                            }
                            self.project_settings.local_files.reload(&self.storage)

                        }
                        4 => {
                            for node in &self.nodes {
                                self.backup_data.retain(|x| x.id != node.id());
                            }
                            for data in &self.backup_data {
                                self.nodes.push(data.restore_node());
                            }
                        }
                        5 => {
                            if user_settings.install_ffmpeg {
                            ffmpeg_sidecar::download::auto_download().unwrap();
                        };
                        }
                        a => {
                            unreachable!("There is no loading step: {a}")
                        }
                    }
                    self.loading += 1;
                });
            return;
        }

        // if the user is running a batch file edit
        // this should probably be moved into its own function
        if self.project_settings.batch_files.run {
            ui.window("running batch")
            .position([window_size.x*0.5,window_size.y*0.5], imgui::Condition::Always)
            .position_pivot([0.5,0.5])
            .size([window_size.x * 0.5, window_size.y * 0.5], imgui::Condition::Always)
            .build(|| {
                ui.text(format!("{}",self.project_settings.batch_files.files[self.project_settings.batch_files.index].name()));
                ui.text(format!("{}",self.project_settings.batch_files.save_path.join(self.project_settings.batch_files.files[self.project_settings.batch_files.index].name()).with_extension(self.project_settings.batch_files.files[self.project_settings.batch_files.index].type_()).display()));
                ui.text(format!("{}/{}",self.project_settings.batch_files.index,self.project_settings.batch_files.files.len()));
            });
            self.run_batch(renderer);
            return;
        }

        let mut sidebar_params = SidebarParams {
            menu_bar_size:  ui.item_rect_size(),
            ..Default::default()
        };


        // start by rendering the menu bar. 
        ui.main_menu_bar(|| {
            
            ui.menu("project", || {
                if ui.menu_item("open folder") {
                    open::that_detached(&self.path);
                }
                if ui.menu_item("open resource root") {
                    open::that_detached(&self.path.join("root"));
                }
                if ui.menu_item("manual save") {
                    self.save();
                }
                if ui.menu_item("reload project") {
                    self.loading = 0;
                }
            });
            
            ui.menu("settings", || {
            if ui.menu_item("user settings") {
                self.open_settings = true;
            }
            });

            ui.menu("debug", || {
                if ui.menu_item("debug imgui") {
                    self.metrics = !self.metrics;
                }
                if ui.menu_item("debug memory") {
                    self.storage.max_lines_of_text = 1;
                    self.storage.show_debug_window = !self.storage.show_debug_window;
                }
            });
            ui.menu("feedback", || {
                if ui.menu_item("star on github") {
                    open::that("https://github.com/ollielynas/reanimator");
                }
                if ui.menu_item("bug report") {
                    let bug_body = "Try to include info like:%0A- how to replicate the issue%0A- what the issue was%0A-screenshots%0A- error messages%0A- an exported version of the project%A0%0Adon't forget a title".replace(" ", "%20");
                    open::that(format!("https://github.com/ollielynas/reanimator/issues/new?labels=bug&title=[bug]%20v{}-&body=\"{}\"", cargo_crate_version!(), bug_body));
                }
                if ui.menu_item("request feature") {
                    open::that("https://github.com/ollielynas/reanimator/issues/new?title=[feature%20request]&body=don't%20forget%20a%20title");
                }
                if ui.menu_item("spelling mistake") {
                    open::that("https://github.com/ollielynas/reanimator/issues/new?title=[important]&body=please%20be%20thorough%20in%20your%20description%20of%20the%20error");
                }
                if ui.menu_item("google form") {
                    open::that("https://docs.google.com/forms/d/e/1FAIpQLSfBSZZc8oqVrxUUfmNsEjvmKtE3RKdPIok7WvWQWk5S3mW4XQ/viewform?usp=sf_link");
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("if you don't have/don't want to create a github account you can use this google form");
                }
            });
        });


        
        let wheel_delta = ui.io().mouse_wheel;

        let mut params: RenderNodesParams = RenderNodesParams {
            duplicate_node: None,
            move_delta: ui.io().mouse_delta,
            size_array,
            moving: false,
            connection_hash: self.nodes.len() as u64
                + calculate_hash(
                    &<HashMap<String, String> as Clone>::clone(&self.connections)
                        .into_iter()
                        .collect::<Vec<(String, String)>>(),
                ),
            scale_changed: wheel_delta.abs() > 0.0,
            node_pos_map: HashMap::new(),
            time_list: vec![],
            delete_node: None,
        };


        let window_params = vec![
            ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0)),
            ui.push_style_var(imgui::StyleVar::WindowRounding(0.0)),
            // ui.push_style_var(imgui::StyleVar::),
        ];

        ui.window("tabs")
            // .draw_background(false)
            .bring_to_front_on_focus(false)
            .position([sidebar_params.left_sidebar_width, sidebar_params.menu_bar_size[1]], imgui::Condition::Always)
            .size_constraints([window_size.x,-1.0], [window_size.x, -1.0])
            .no_decoration()
            .build(|| {
                if let Some(_tab_bar) = ui.tab_bar("top tab bar") {
                    if let Some(_item) = ui.tab_item("nodes") {
                        self.edit_tab = EditTab::Nodes;
                    }
                    if let Some(_item) = ui.tab_item("batch edit") {
                        self.edit_tab = EditTab::BatchFileEdit;
                    }
                    if let Some(_item) = ui.tab_item("project resources") {
                        self.edit_tab = EditTab::ProjectRes;
                    }
                }
                sidebar_params.menu_bar_size[1] += ui.window_size()[1] - 7.0;
            });
        
        for i in window_params {
            i.end();
        }


        self.render_sidebar(&mut params, ui, &mut sidebar_params, user_settings);
        self.storage.debug_window(ui, &mut params);

        match self.edit_tab {
            EditTab::Nodes => {},
            EditTab::BatchFileEdit => {
                self.render_batch_edit(ui, &mut sidebar_params, user_settings);
            },
            EditTab::ProjectRes => {
                self.render_local_files(ui, &mut sidebar_params, user_settings);
            },
        }

        if self.edit_tab != EditTab::Nodes {
            return;
        }



        let mut run_id = String::new();
        for node in &self.nodes {
            if node.type_() == NodeType::CoverWindow {
                let a: Option<&CoverWindowNode> =
                    (*node).as_any().downcast_ref::<CoverWindowNode>();
                if let Some(cover_node) = a {
                    if cover_node.type_() == NodeType::CoverWindow && cover_node.render {
                        run_id = node.id();
                    }
                }
            }
        }

        if run_id != String::new() {
            self.storage.time = ui.time();
            self.run_nodes(renderer);

            // log::info!("ran");
        }
        for node in &mut self.nodes {
            if node.type_() == NodeType::CoverWindow {
                let a: Option<&mut CoverWindowNode> =
                    (*node).as_any_mut().downcast_mut::<CoverWindowNode>();
                if let Some(cover_node) = a {
                    if cover_node.render && run_id == cover_node.id() {
                        if cover_node.render(ui, window, &mut self.storage, renderer) {
                            // ui.text("text");
                            
                            return;
                        } else {
                            window.set_cursor_hittest(true);
                            window.set_decorations(true);
                            window.set_resizable(true);
                            window.set_window_level(
                                imgui_winit_support::winit::window::WindowLevel::default(),
                            );
                        }
                    }
                }
            }
        }





        // log::info!("{:?}", ui.mouse_drag_delta());

        if let Some(_popup_menu) = ui.begin_popup_context_window() {
            if ui.menu_item("new node") {
                sidebar_params.new_node_popup = true;
            }
        }

        // self.new_node_menu(ui);

        if ui.is_window_focused() {
            self.selected = None;
        }

        let mouse_pos = ui.io().mouse_pos;

        // ui.get_foreground_draw_list().add_line(mouse_pos, [mouse_pos[0]+params.move_delta[0]*10.0 , mouse_pos[1] +params.move_delta[1] * 10.0], [1.0,0.0,1.0,1.0]).thickness(1.0).build();

        let node_window_vars = [
            ui.push_style_var(imgui::StyleVar::ItemSpacing([
                3.0 * self.scale,
                3.0 * self.scale,
            ])),
            ui.push_style_var(imgui::StyleVar::WindowPadding([
                10.0 * self.scale,
                10.0 * self.scale,
            ])),
            ui.push_style_var(imgui::StyleVar::FramePadding([
                5.0 * self.scale,
                5.0 * self.scale,
            ])),
            ui.push_style_var(imgui::StyleVar::WindowMinSize([
                5.0 * self.scale,
                5.0 * self.scale,
            ])),
        ];

        // move_delta[0] /= self.scale * -1.0;
        // move_delta[1] /= self.scale * -1.0;

        if ui.is_mouse_down(imgui::MouseButton::Left)
            && ui.is_mouse_dragging(imgui::MouseButton::Left)
        {
            params.moving = true;
            // log::info!("")
        }

        self.render_node(ui, &mut params, renderer);

        for var in node_window_vars {
            var.end();
        }

        if let Some(mut d) = params.duplicate_node {
            d.set_xy(d.x() + 10.0, d.y() + 10.0);
            d.set_id(random_id());
            self.nodes.push(d);
        }

        if let Some(kill) = params.delete_node {
            self.nodes.remove(kill);
        }

        self.recenter = false;

        let draw_list = ui.get_background_draw_list();
        if user_settings.dots {
            self.render_background(ui, &draw_list);
        };

        let mouse_pos = ui.io().mouse_pos;
        match (
            self.selected_input.clone(),
            self.selected_output.clone(),
            mouse_pos,
        ) {
            (None, Some(a), m) => {
                if let Some(pos) = params.node_pos_map.get(&a) {
                    let mut pos = pos.clone();
                    // if pos == ImVec2::new(PI, PI) {
                    //     pos = ImVec2::new(size_array[0], m[1]);
                    // }
                    let diff = (m[0] - pos.x).abs();
                    draw_list
                        .add_bezier_curve(
                            m,
                            [m[0] - diff * 0.3, m[1]],
                            [pos.x + diff * 0.3, pos.y],
                            [pos.x, pos.y],
                            ImColor32::BLACK,
                        )
                        .build();
                }
            }
            (Some(a), None, m) => {
                if let Some(pos) = params.node_pos_map.get(&a) {
                    let diff = (m[0] - pos.x).abs();

                    draw_list
                        .add_bezier_curve(
                            [pos.x, pos.y],
                            [pos.x - diff * 0.3, pos.y],
                            [m[0] + diff * 0.3, m[1]],
                            [m[0], m[1]],
                            ImColor32::BLACK,
                        )
                        .build();
                }
                self.connections.remove(&a);
            }
            (Some(a), Some(b), _) => {
                self.connections.insert(a, b);

                self.selected_input = None;
                self.selected_output = None;
            }
            (None, None, _) => {}
        }

        if self.selected_input.is_some() || self.selected_output.is_some() {
            if ui.is_any_mouse_down() && !ui.is_any_item_hovered() {
                self.selected_input = None;
                self.selected_output = None;
            }
        }

        for (a, b) in &self.connections {
            if let Some(pos2) = params.node_pos_map.get(a) {
                if let Some(pos) = params.node_pos_map.get(b) {
                    let texture_input = self.storage.get_texture(b).is_some();
                    let text_input = self.storage.get_text(b).is_some();
                    let dif = (pos.x - pos2.x).abs();
                    draw_list
                        .add_bezier_curve(
                            [pos.x, pos.y],
                            [(pos.x + dif * 0.3), pos.y],
                            [(pos2.x - dif * 0.3), pos2.y],
                            [pos2.x, pos2.y],
                            if text_input {
                                [0.0, 0.5, 0.0, 1.0]
                            } else if texture_input {
                                [0.0, 0.0, 0.5, 1.0]
                            } else {
                                [0.0, 0.0, 0.0, 1.0]
                            },
                        )
                        .thickness(
                            if texture_input || text_input {
                                3.0
                            } else {
                                2.0
                            } * self.scale,
                        )
                        .build();
                }
            }
        }
        // });

        if self.project_settings.render_ticker
            && self.render_ticker_timer.elapsed().as_secs_f32() > 1.0
        {
            self.render_ticker_timer = Instant::now();
            params.time_list.push(ui.time());
        }

        let mut before = glium::debug::TimestampQuery::new(&self.storage.display);
        let mut after = glium::debug::TimestampQuery::new(&self.storage.display);
        let mut first = true;
        // params.time_list = vec![0.0];
        for t in params.time_list {
            // log::info!("t");
            if first {
                before = glium::debug::TimestampQuery::new(&self.storage.display);
            }
            let before_run_nodes = Instant::now();
            self.storage.time = t;
            self.run_nodes(renderer);
            self.total_frame_time = before_run_nodes.elapsed().as_secs_f32();
            if first {
                after = glium::debug::TimestampQuery::new(&self.storage.display);
            }
            first = false;
        }

        let un_round = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));

        
        self.advanced_color_picker.render(ui);

        if self.open_settings {
            user_settings.settings_window(ui, &mut self.open_settings, &self.storage.fonts);
        }

        let mut edit_window_pos: [f32; 2] = [0.0; 2];

        ui.window("Edit Node Properties")
            .collapsible(false)
            .position_pivot([0.0, 1.0])
            .position(
                [sidebar_params.left_sidebar_width - 1.0, size_array[1] + 3.0],
                imgui::Condition::Always,
            )
            .size_constraints(
                [size_array[0] - sidebar_params.left_sidebar_width + 3.0, 0.0],
                [size_array[0] - sidebar_params.left_sidebar_width + 3.0, size_array[1]],
            )
        
            .build(|| {
                if ui.is_window_hovered() && ui.is_mouse_down(imgui::MouseButton::Left) {
                    params.moving = false;
                }
                if ui.is_window_hovered() {
                    params.scale_changed = false;
                }
                match self.node_edit {
                    Some(a) if self.nodes.len() > a => {
                        self.nodes[a].edit_menu_render(ui, renderer, &self.storage);
                    }
                    _ => ui.text("no node has been selected"),
                }
                edit_window_pos = ui.window_pos();
            });
        un_round.end();

        if self.metrics {
            ui.show_metrics_window(&mut self.metrics);
        }

        ui.window("frame time")
            .size_constraints([window_size.x * 0.5, -1.0], [window_size.x * 0.5, -1.0])
            .no_decoration()
            .draw_background(false)
            // .no_inputs()
            // .flags(WindowFlags:)
            .position(edit_window_pos, imgui::Condition::Always)
            .position_pivot([0.0, 1.0])
            .build(|| {
                match (after, before) {
                    (Some(after), Some(before)) => {
                        let elapsed = after.get() - before.get();
                        // log::info!("{}", elapsed);
                        if elapsed > 100000 {
                            self.total_gpu_frame_time = (elapsed as f64 / 10.0_f64.powi(9)) as f32;
                        }
                    }
                    _ => (),
                }

                ui.text(format!(
                    "[CPU/GPU]: [{:.2}/{:.2}] ms",
                    self.total_frame_time * 1000.0,
                    self.total_gpu_frame_time * 1000.0,
                ));
            });

        if ui.mouse_cursor() != Some(imgui::MouseCursor::Arrow)
            && ui.mouse_cursor() != Some(imgui::MouseCursor::Hand)
        {
            params.moving = false;
        }

        if ui.is_any_item_hovered() || ui.is_any_item_active() {
            params.moving = false;
        }

        if params.scale_changed {
            let new_scale = (self.scale * (1.1_f32.powf(wheel_delta))).clamp(0.05, 2.0);
            let scale_delta = new_scale / self.scale;
            let mouse_pos_graph = screen_to_graph_pos(mouse_pos, self.graph_offset, self.scale);
            for i in [1, 0] {
                self.graph_offset[i] =
                    ((scale_delta - 1.0) * mouse_pos_graph[i] + self.graph_offset[i]) / scale_delta;
            }

            self.scale = new_scale;
            // for i in [1,0] {
            //     self.graph_offset[0];
        }

        if params.moving {
            ui.set_mouse_cursor(Some(imgui::MouseCursor::Hand));
            self.graph_offset[0] -= params.move_delta[0] / self.scale;
            self.graph_offset[1] -= params.move_delta[1] / self.scale;
        }

        // for i in self.nodes {

        // self.run_nodes();

        // }

        // ui.show_demo_window(&mut true);

        // ui.show_user_guide();
    }

    pub fn run_nodes(&mut self, renderer: &mut Renderer) {
        self.run_nodes_on_io_arrays(renderer, RawImage2d::from_raw_rgb(vec![], (0,0)), &mut RawImage2d::from_raw_rgb(vec![], (0,0)));
    }
    pub fn run_nodes_on_io_arrays(&mut self, renderer: &mut Renderer, input: RawImage2d<u8>, output: &mut RawImage2d<u8>) {
        self.storage.reset();
        self.node_speeds.clear();

        let mut do_io = input.data.len() > 0 
        && self.project_settings.generic_io.input_id.is_some()
        && self.project_settings.generic_io.output_id.is_some();

        let input_node_id = <Option<String> as Clone>::clone(&self.project_settings.generic_io.input_id).unwrap_or_default();
        let output_node_id = <Option<String> as Clone>::clone(&self.project_settings.generic_io.output_id).unwrap_or_default();


        let mut input_texture_id = String::new();
        let mut output_texture_id = String::new();

        if do_io {
        for node in &self.nodes {
            if node.id() == input_node_id {
                input_texture_id = node.output_id(node.outputs()[0].clone());
            }
            if node.id() == output_node_id {
                let input_id = node.input_id(node.inputs()[0].clone());
                let get_output = match self.connections.get(&input_id) {
                    Some(a) => a.to_owned(),
                    None => {do_io = false;String::new()},
                };
                output_texture_id = get_output;
            }
        }
        }


        if do_io {
            self.storage.create_and_set_texture(input.width, input.height, input_texture_id.clone());
            self.storage.get_texture(&input_texture_id).unwrap().write(Rect {
                left: 0,
                bottom: 0,
                width: input.width,
                height: input.height,
            }, input);
        }

        // log::info!("run");

        let connection_hash = self.nodes.len() as u64
            + calculate_hash(
                &<HashMap<String, String> as Clone>::clone(&self.connections)
                    .into_iter()
                    .collect::<Vec<(String, String)>>(),
        );

        let mut node_indices: HashMap<String, usize> = HashMap::new();

        for (i, n) in self.nodes.iter().enumerate() {
            node_indices.insert(n.id(), i);
        }

        if connection_hash != self.node_run_order.0 {
            let mut node_graph: HashMap<String, Vec<String>> = HashMap::new();

            for (parent, child) in &self.connections {
                let parent_id = parent.split("-").collect::<Vec<&str>>()[1];
                let child_id = child.split("-").collect::<Vec<&str>>()[1];

                if let Some(a) = node_graph.get_mut(parent_id) {
                    a.push(child_id.to_string());
                } else {
                    node_graph.insert(parent_id.to_string(), vec![child_id.to_string()]);
                }
            }

            let mut colors: HashMap<String, u8> = HashMap::new();
            let mut order: Vec<String> = vec![];

            fn dfs(
                id: String,
                colors: &mut HashMap<String, u8>,
                order: &mut Vec<String>,
                node_graph: &mut HashMap<String, Vec<String>>,
            ) {
                let color = colors.get(&id).unwrap_or(&0);
                match color {
                    0 => {
                        colors.insert(id.clone(), 1);
                        let mut c = vec![];
                        for connection in node_graph.get(&id.clone()).unwrap_or(&vec![]) {
                            c.push(connection.to_string());
                        }
                        for a in c {
                            dfs(a, colors, order, node_graph);
                        }
                        colors.insert(id.clone(), 1);
                        order.push(id.clone());
                        return;
                    }
                    1 => {
                        log::info!("loop :/");
                        return;
                    }
                    2 => {
                        return;
                    }
                    _ => {
                        unreachable!("oops :(")
                    }
                }
            }

            let mut outputs: Vec<String> = vec![];
            if outputs.len() == 0 {
                for (_, n) in self.nodes.iter().enumerate() {
                    if matches!(
                        n.type_(),
                        NodeType::Output //| NodeType::Output
                        | NodeType::CoverWindow //| NodeType::Output
                        
                    ) {
                        outputs.push(n.id());
                    }
                }
            }

            for out in outputs {
                dfs(out, &mut colors, &mut order, &mut node_graph);
            }
            self.node_run_order = (connection_hash, order);
        }

        for id in &self.node_run_order.1 {
            if let Some(index) = node_indices.get(id) {
                if self.nodes.len() > *index {
                    let now = Instant::now();

                    let worked = 
                    if !do_io || self.nodes[*index].id() != input_node_id {
                    self.nodes[*index].run(
                        &mut self.storage,
                        self.connections.clone(),
                        renderer,
                    )}
                    else
                    {true};
                    if worked {
                        self.node_speeds.insert(id.to_string(), now.elapsed());
                    } else {
                        self.node_speeds.remove(id);
                    }
                }
            }
        }

        if let Some(texture) = self.storage.get_texture(&output_texture_id) {
            *output = RawImage2d::from_raw_rgba(
                texture.read_to_pixel_buffer().read().unwrap().iter().flat_map(|(r,g,b,a)| [r,g,b,a]).copied().collect_vec(),
                texture.dimensions());
        }
    }

    pub fn new_node_menu(&mut self, ui: &Ui, user_settings: &UserSettings) -> bool {
        let mut group: HashMap<String, Option<TreeNodeToken>> = HashMap::new();
        let size_array = ui.io().display_size;

        let mut open = false;

        ui.modal_popup_config("Add Node")
            // .resizable(false)
            .build(|| {
                open = true;
                ui.columns(2, "select new node col", true);
                ui.input_text("search", &mut self.node_search_string).build();
                let mut node_order =  (0..self.new_node_types.len()).collect::<Vec<usize>>();
                if ui.is_item_edited() || true {
                    let alg: Levenshtein = Levenshtein::default();
                    node_order.retain(|n| (alg.for_str(&self.new_node_types[*n].name().to_lowercase(), &self.node_search_string.to_lowercase()).ndist())  < 0.9);
                    node_order.sort_by_cached_key(|n| (alg.for_str(&self.new_node_types[*n].name().to_lowercase(), &self.node_search_string.to_lowercase()).ndist() * 10000.0) as i32);
                }

                let avil = ui.content_region_avail();
                ui.child_window("child window")
                    .size([0.0, -ui.calc_text_size("|")[1] * 1.5])
                    .build(|| {
                        if self.node_search_string != String::new() {
                            for n in node_order {
                                let mut name = self.new_node_types[n].name();
                                if self.new_node_types[n].type_().disabled() {
                                    name += " (Debug Only)"
                                }
                                ui.radio_button(name, &mut self.selected_node_to_add, n);

                            }
                        }else {
                        for n in 0..self.new_node_types.len() {
                            #[cfg(not(debug_assertions))]
                            {
                                if self.new_node_types[n].type_().disabled() {
                                    continue;
                                }
                            }
                            let mut invalid_tree: Vec<String> = vec![];
                            for k in group.keys() {
                                if !self.new_node_types[n].path().contains(&&k.as_str()) {
                                    invalid_tree.push(k.to_string());
                                }
                            }
                            for invalid in invalid_tree {
                                if let Some(Some(a)) = group.remove(&invalid) {
                                    a.end();
                                }
                            }
                            for p in self.new_node_types[n].path() {
                                if !group.contains_key(p) {
                                    group.insert(p.to_owned(), ui.tree_node(p));
                                }
                                if !matches!(group.get(p), Some(Some(_))) {
                                    break;
                                }
                            }

                            if self.new_node_types[n]
                                .path()
                                .iter()
                                .all(|x| matches!(group.get(&x.to_string()), Some(Some(_))))
                            {
                                let mut name = self.new_node_types[n].name();
                                if self.new_node_types[n].type_().disabled() {
                                    name += " (Debug Only)"
                                }
                                ui.radio_button(name, &mut self.selected_node_to_add, n);

                            }
                        }
                    }

                        for i in group.drain() {
                            if let (_, Some(a)) = i {
                                a.end();
                            }
                        }
                    });

                if let Some(new_node) = self.new_node_types.get(self.selected_node_to_add) {
                    if ui.button("add") {
                        let mut new_node2 = self.new_node_types[self.selected_node_to_add]
                            .type_()
                            .new_node();
                        // let center = screen_to_graph_pos(ui.cursor_screen_pos(), self.graph_offset, self.scale);
                        let center = [size_array[0] * 0.5, size_array[1] * 0.3];
                        new_node2.set_xy(center[0], center[1]);
                        self.nodes.push(new_node2);

                        ui.close_current_popup();
                    }
                    ui.same_line()
                }
                if ui.button("cancel") {
                    ui.close_current_popup();
                }

                ui.next_column();
                // ui.text(format!("{:#?}", group_open));

                if self.new_node_types.len() > self.selected_node_to_add {
                    ui.set_window_font_scale(1.3);
                    ui.text(self.new_node_types[self.selected_node_to_add].name());
                    ui.set_window_font_scale(1.0);
                    #[cfg(debug_assertions)] {
                        let v = vec![0.0];
                        let array = user_settings.node_speed.get(&self.new_node_types[self.selected_node_to_add].name()).unwrap_or(&v);
                        ui.text(format!("{}",array.iter().sum::<f32>() / array.len() as f32 ));
                    }
                    self.new_node_types[self.selected_node_to_add].description(ui);
                } else {
                    ui.text("no node has been selected");
                }

                // self.new_node_types[n].;
            });

        return open;
    }

    pub fn drop_file(&mut self, path: PathBuf, ui: &Ui) {
        let binding = OsString::new();
        let ext = path.extension().unwrap_or(&binding).to_str().unwrap_or("");

        let [mut x, mut y] = ui.io().mouse_pos;
        x = 100.0;
        y = 100.0;
        log::info!("droped {:?}", path);
        match ext {
            "gif" => {
                log::info!("gif");
                let mut node = NodeType::LoadGif.new_node();
                let a: Option<&mut LoadGifNode> =
                    (*node).as_any_mut().downcast_mut::<LoadGifNode>();
                if let Some(g_node) = a {
                    g_node.path = Some(path)
                }
                node.set_xy(x, y);
                self.nodes.push(node);
            }
            "png" | "jpg" | "jepg" | "webp" | "tiff" | "tif" | "tga" | "bmp" | "ico" | "hdr"
            | "pbm" | "pam" | "ppm" | "pgm" | "ff" => {
                let mut node = NodeType::LoadImageType.new_node();
                let a: Option<&mut LoadImage> = (*node).as_any_mut().downcast_mut::<LoadImage>();
                if let Some(g_node) = a {
                    g_node.path = Some(path)
                }
                node.set_xy(x, y);
                self.nodes.push(node);
            }
            _ => {}
        }
    }
}


    

pub fn graph_to_screen_pos(mut pos: [f32; 2], graph_offset: [f32; 2], scale: f32) -> [f32; 2] {
    for i in [0, 1] {
        pos[i] -= graph_offset[i];
        pos[i] *= scale;
    }
    return pos;
}
pub fn screen_to_graph_pos(mut pos: [f32; 2], graph_offset: [f32; 2], scale: f32) -> [f32; 2] {
    for i in [0, 1] {
        pos[i] /= scale;
        pos[i] += graph_offset[i];
    }
    return pos;
}
