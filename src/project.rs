use glium::{program, BlitTarget, Display, Program, Surface};
use imgui::drag_drop::PayloadIsWrongType;
use imgui::{Style, WindowHoveredFlags};
use imgui::{sys::ImVec2, ImColor32, TreeNodeToken, Ui};
use imgui_glium_renderer::Renderer;
use platform_dirs::AppDirs;
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

use crate::node::random_id;
use crate::nodes::load_gif::LoadGifNode;
use crate::nodes::load_image::LoadImage;
use crate::{project, project_settings};
use crate::project_settings::ProjectSettings;
use crate::render_nodes::RenderNodesParams;
use crate::{
    advanced_color_picker::AdvancedColorPicker, history_tracker::Snapshot, node,
    nodes::node_enum::*,
};
use crate::{
    node::MyNode,
    nodes::{self, image_io::*},
    storage::Storage,
    user_info::{self, UserSettings},
};
use rfd::FileDialog;

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

const MAX_LOADING: i32 = 3;

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
    metrics: bool,
    new_node_types: Vec<Box<dyn MyNode>>,
    pub path: PathBuf,
    pub node_speeds: HashMap<String, Duration>,
    /// which node should have its edit window open
    pub node_edit: Option<usize>,
    pub node_run_order: (u64, Vec<String>),
    /// used by the "add node" model popup to track which node has been selected
    selected_node_to_add: usize,
    pub return_to_home_menu: bool,
    pub display_history: bool,
    pub snapshots: Vec<Snapshot>,
    pub selected_snapshot: i32,
    recenter: bool,
    advanced_color_picker: AdvancedColorPicker,
    pub pop_out_edit_window: HashMap<String, bool>,
    total_frame_time: f32,
    loading: i32,
    render_ticker_timer: Instant,
    open_settings: bool,
    project_settings: ProjectSettings,
}

impl Project {
    // pub fn calculate_hash(&self) -> u64 {
    //     // for node in self.nodes {
    //     //     // self.storage.calculate_hash(&node.);
    //     // }
    // }

    pub fn new(path: PathBuf, display: Display<WindowSurface>) -> Project {
        // println!("{:?}", fs::create_dir_all(path.join("nodes")));

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
            loading: 0,
            project_settings: ProjectSettings::default(),
            render_ticker_timer: Instant::now(),
            open_settings: false,
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
    pub fn save(&self) -> Result<(), SavefileError> {
        // println!("{:?}", self.path);
        // println!("{:?}", self.path.join("connections.bin"));

        if self.loading <= MAX_LOADING {
            return Ok(());
        }

        fs::create_dir_all(self.path.clone())?;

        savefile::save_file(self.path.join("connections.bin"), 0, &self.connections)?;
        savefile::save_file(self.path.join("project_settings.bin"), 0, &self.project_settings)?;
        let _ = fs::remove_dir_all(self.path.join("nodes"));
        fs::create_dir_all(self.path.join("nodes"))?;
        for node in &self.nodes {
            fs::create_dir_all(self.path.join("nodes").join(node.name()))?;
            node.save(self.path.join("nodes"))?;
        }

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

    pub fn render(&mut self, ui: &Ui, user_settings: &mut UserSettings, renderer: &mut Renderer) {
        let size_array = ui.io().display_size;
        let window_size = ImVec2::new(size_array[0], size_array[1]);

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
                        ((self.loading as f32 / MAX_LOADING as f32) * 100.0
                            // - fastrand::f32() * (100.0 / MAX_LOADING as f32)
                        )
                            .clamp(0.0, 100.0)
                            .round()
                    ));
                    ui.set_window_font_scale(1.0);
                    println!("loading step: {}", self.loading);

                    if self.loading != 0 {
                        // sleep(Duration::from_secs_f32(0.25));
                    }

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
                        _ => {}
                    }

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

                            nodes[0].set_xy(150.0, 0.0);
                            nodes[1].set_xy(0.0, 0.0);

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
                                                    println!("loaded node");
                                                };
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("project not found");
                            }
                        }
                        2 => {
                            self.recenter_nodes(ui);

                            self.run_nodes(renderer);
                            self.run_nodes(renderer);
                        }
                        3 => {
                            self.save();
                        }
                        a => {
                            unreachable!("There is no loading step: {a}")
                        }
                    }
                    self.loading += 1;
                });
            return;
        }

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

        // Context::set_pixels_per_point(&self, pixels_per_point)
        // self.node_edit = None;
        // hash


        ui.main_menu_bar(|| {
            ui.text(self.path.as_os_str().to_str().unwrap());
            // ui.set_window_font_scale(0.9);
            // ui.checkbox(";", &mut self.render_ticker);
        });

        let menu_bar_size = ui.item_rect_size();


        let mut new_node_popup = false;

        // unsafe { ui.style().scale_all_sizes(0.5) };
        // let mut style = ui.style().
        // ui.show_style_editor();
        // ui.show_default_style_editor();
        // renderer.render(target, draw_data)

                // println!("{:?}", ui.mouse_drag_delta());

                if let Some(_popup_menu) = ui.begin_popup_context_window() {
                    if ui.menu_item("new node") {
                        new_node_popup = true;
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

                if  ui.is_mouse_down(imgui::MouseButton::Left)
                        && ui.is_mouse_dragging(imgui::MouseButton::Left)
                    {
                        params.moving = true;
                        // println!("")
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
                        if let Some(pos) =  params.node_pos_map.get(&a) {
                            let mut pos = pos.clone();
                            if pos == ImVec2::new(PI, PI) {
                                pos = ImVec2::new(size_array[0], m[1]);
                            }
                            draw_list
                                .add_bezier_curve(
                                    m,
                                    [(m[0] + pos.x) * 0.5, m[1]],
                                    [(m[0] + pos.x) * 0.5, pos.y],
                                    [pos.x, pos.y],
                                    ImColor32::BLACK,
                                )
                                .build();
                        }
                    }
                    (Some(a), None, m) => {
                        if let Some(pos) = params.node_pos_map.get(&a) {
                            draw_list
                                .add_bezier_curve(
                                    [pos.x, pos.y],
                                    [(m[0] + pos.x) * 0.5, pos.y],
                                    [(m[0] + pos.x) * 0.5, m[1]],
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
                    if let Some(pos2) =  params.node_pos_map.get(a) {
                        if let Some(pos) =  params.node_pos_map.get(b) {

                            
                            let texture_input = self.storage.get_texture(b).is_some();
                            let text_input = self.storage.get_text(b).is_some();

                            draw_list
                                .add_bezier_curve(
                                    [pos.x, pos.y],
                                    [(pos.x + pos2.x) * 0.5, pos.y],
                                    [(pos.x + pos2.x) * 0.5, pos2.y],
                                    [pos2.x, pos2.y],
                                    if text_input {[0.0,0.5,0.0,1.0]} else if texture_input {[0.0, 0.0, 0.5,1.0]} else {[0.0,0.0,0.0,1.0]},
                                )
                                .thickness(if texture_input || text_input {3.0} else {2.0} * self.scale)
                                .build();
                        }
                    }
                }
            // });

        if self.project_settings.render_ticker && self.render_ticker_timer.elapsed().as_secs_f32() > 1.0 {
            self.render_ticker_timer = Instant::now();
            params.time_list.push(ui.time());
        }

        for t in params.time_list {
            let before_run_nodes = Instant::now();
            self.storage.time = t;
            self.run_nodes(renderer);
            self.total_frame_time = before_run_nodes.elapsed().as_secs_f32();
        }

        let mut left_sidebar_width = 0.0;
        let un_round = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));

        ui.window("sidebar")
            .no_decoration()
            .focused(self.recenter)
            .position([0.0, menu_bar_size[1] - 1.0], imgui::Condition::Always)
            .size_constraints([0.0, window_size.y], [window_size.x * 0.4, window_size.y])
            .resizable(true)
            // .always_auto_resize(true)
            .build(|| {
                let sidebar_things = vec![ui.push_style_var(imgui::StyleVar::FrameBorderSize(0.0))];
                let sidebar_col_things = vec![
                    ui.push_style_color(imgui::StyleColor::Button, [1.0, 1.0, 1.0, 0.0]),
                    ui.push_style_color(imgui::StyleColor::BorderShadow, [0.0, 0.0, 1.0, 1.0]),
                    // ui.push_style_color(imgui::StyleColor::ButtonHovered, ui.style_color(S)),
                ];

                if ui.is_window_hovered() && ui.is_mouse_down(imgui::MouseButton::Left) {
                    params.moving = false;
                }

                // Style::use_light_colors(&mut self)
                if ui.button("add node") || new_node_popup {
                    ui.open_popup("Add Node");
                }

                if ui.button("color picker") {
                    self.advanced_color_picker.open = !self.advanced_color_picker.open;
                };

                if user_settings.history {
                    if ui.button("timeline") {
                        self.display_history = !self.display_history;
                    }
                }

                if ui.button("recenter") {
                    self.recenter_nodes(ui);
                    self.recenter = true;
                }

                if ui.button("copy file to local res") {
                    fs::create_dir_all(self.path.join("res"));
                    if let Some(paths) = FileDialog::new().pick_files() {
                        for path in paths {
                            let _ = fs::copy(
                                path.clone(),
                                self.path.join("res").join(
                                    (&path)
                                        .file_name()
                                        .unwrap_or(&OsString::new())
                                        .to_str()
                                        .unwrap(),
                                ),
                            );
                        }
                    }
                }

                ui.separator();

                if ui.button("debug") {
                    self.metrics = !self.metrics;
                }
                if ui.button("debug mem") {
                    self.storage.show_debug_window = !self.storage.show_debug_window;
                }

                if ui.button("settings") {
                    self.open_settings = true;
                }
                if ui.button("save") {
                    println!("save button, {:?}", self.save());
                }
                
                ui.separator();

                ui.checkbox("auto update", &mut self.project_settings.render_ticker);
                
                ui.separator();
                
                if ui.button("return home") {
                    self.save();
                    self.return_to_home_menu = true;
                }
                if self.display_history {
                    self.history_window(ui);
                }

                left_sidebar_width = ui.window_size()[0];
                for i in sidebar_col_things {
                    i.end();
                }
                for i in sidebar_things {
                    i.end();
                }

                if self.new_node_menu(ui) {
                    params.moving = false;
                    params.scale_changed = false;

                }

                if ui.is_window_hovered() {
                    params.scale_changed = false;
                }

            
            });

        self.storage.debug_window(ui);
        self.advanced_color_picker.render(ui);

        if self.open_settings {
            user_settings.settings_window(ui, &mut self.open_settings, &self.storage.fonts);
        }

        let mut edit_window_pos: [f32; 2] = [0.0; 2];

        ui.window("edit_node")
            .collapsible(true)
            .position_pivot([0.0, 1.0])
            .position(
                [left_sidebar_width - 1.0, size_array[1]],
                imgui::Condition::Always,
            )
            .size_constraints(
                [size_array[0] - left_sidebar_width, 0.0],
                [size_array[0] - left_sidebar_width, size_array[1]],
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
                        self.nodes[a].edit_menu_render(ui, renderer);
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
            .no_inputs()
            .position(edit_window_pos, imgui::Condition::Always)
            .position_pivot([0.0, 1.0])
            .build(|| {
                ui.text(format!(
                    "frame time: {:.4}ms",
                    self.total_frame_time * 1000.0
                ));
            });


            if ui.mouse_cursor() != Some(imgui::MouseCursor::Arrow) && ui.mouse_cursor() != Some(imgui::MouseCursor::Hand) {
                params.moving = false;
            }


            if ui.is_any_item_hovered() 
            || ui.is_any_item_active()
            {
                params.moving = false;
            }

            if params.scale_changed {
                let new_scale = (self.scale * (1.1_f32.powf(wheel_delta))).clamp(0.05, 2.0);
                let scale_delta = new_scale/self.scale;
                let mouse_pos_graph = screen_to_graph_pos(mouse_pos, self.graph_offset, self.scale);
                for i in [1, 0] {
                    self.graph_offset[i] = ((scale_delta -1.0)*mouse_pos_graph[i]+self.graph_offset[i])/scale_delta;
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
        self.storage.reset();
        self.node_speeds.clear();

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
                        println!("loop :/");
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
            for (_, n) in self.nodes.iter().enumerate() {
                if matches!(
                    n.type_(),
                    NodeType::Output //| NodeType::Output
                ) {
                    outputs.push(n.id());
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
                    let worked = self.nodes[*index].run(
                        &mut self.storage,
                        self.connections.clone(),
                        renderer,
                    );
                    if worked {
                        self.node_speeds.insert(id.to_string(), now.elapsed());
                    } else {
                        self.node_speeds.remove(id);
                    }
                }
            }
        }
    }

    fn new_node_menu(&mut self, ui: &Ui) -> bool {
        let mut group: HashMap<String, Option<TreeNodeToken>> = HashMap::new();
        let size_array = ui.io().display_size;

        let mut open = false;

        ui.modal_popup_config("Add Node").build(|| {
            open = true;
            ui.columns(2, "select new node col", true);
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

            for i in group.drain() {
                if let (_, Some(a)) = i {
                    a.end();
                }
            }
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
        println!("droped {:?}", path);
        match ext {
            "gif" => {
                println!("gif");
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
