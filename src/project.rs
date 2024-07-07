use glium::{program, BlitTarget, Display, Program, Surface};
use imgui::{
    color, draw_list,
    internal::RawCast,
    sys::{igGetWindowSize, ImColor, ImVec2},
    ImColor32, Image, Style, TreeNodeToken, Ui, WindowFlags,
};
use imgui_glium_renderer::Renderer;
use platform_dirs::AppDirs;
use std::{
    env::current_exe, f32::{consts::PI, NAN}, hash::{DefaultHasher, Hash, Hasher}, task::ready, time
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

use crate::{history_tracker::Snapshot, nodes::node_enum::*};
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
// #[savefile_derive]
pub struct Project {
    pub storage: Storage,
    pub nodes: Vec<Box<dyn MyNode>>,
    pub selected: Option<usize>,
    graph_offset: (f32, f32),
    scale: f32,
    selected_input: Option<String>,
    selected_output: Option<String>,
    pub connections: HashMap<String, String>,
    metrics: bool,
    new_node_types: Vec<Box<dyn MyNode>>,
    pub path: PathBuf,
    pub node_speeds: HashMap<String, Duration>,
    /// which node should have its edit window open
    node_edit: Option<usize>,
    node_run_order: (u64, Vec<String>),
    /// used by the "add node" model popup to track which node has been selected
    selected_node_to_add: usize,
    pub return_to_home_menu: bool,
    pub display_history: bool,
    pub snapshots: Vec<Snapshot>,
    pub selected_snapshot: i32,
}

impl Project {

    // pub fn calculate_hash(&self) -> u64 {
    //     // for node in self.nodes {
    //     //     // self.storage.calculate_hash(&node.);
    //     // }
    // }


    pub fn project_menu(
        ui: &mut Ui,
        display: &Display<WindowSurface>,
        user_settings: &mut UserSettings,
    ) -> Option<Project> {
        let mut new_project = None;
        let size_array = ui.io().display_size;

        let height = size_array[1] * 0.8;
        let pos = [size_array[0] * 0.5, size_array[1] * 0.5];
        // let size = ui.siz
        ui.window("project selector")
            .collapsible(false)
            .movable(false)
            .position_pivot([0.5, 0.5])
            .bring_to_front_on_focus(false)
            .always_auto_resize(true)
            .size([height * 1.5, height], imgui::Condition::Always)
            .position(pos, imgui::Condition::Always)
            .build(|| {
                ui.columns(2, "col 1324", true);
                ui.input_text("##", &mut user_settings.new_project_name)
                    .auto_select_all(true)
                    .build();
                if ui.button("create new project") {
                    new_project = Some(Project::new(
                        user_settings
                            .project_folder_path
                            .join(user_settings.new_project_name.clone()),
                        display.clone(),
                    ));
                }
                ui.next_column();
                // ui.set_window_font_scale(1.2);
                ui.text("load project");
                // ui.set_window_font_scale(1.0);
                

                ui.window("projects")
                    .always_vertical_scrollbar(true)
                    .position_pivot([0.0, 1.0])
                    // .focused(true)
                    .movable(false)
                    .resizable(false)
                    .collapsible(false)
                    // .draw_background(false)
                    .title_bar(false)
                    .position(
                        [
                            ui.cursor_screen_pos()[0] + 10.0,
                            size_array[1] - height * 0.15,
                        ],
                        imgui::Condition::Always,
                    )
                    .size(
                        [
                            ui.content_region_avail()[0] - 10.0,
                            ui.content_region_avail()[1] - 10.0,
                        ],
                        imgui::Condition::Always,
                    )
                    .build(|| {
                        for project in &user_settings.projects {
                            if ui.button(project.file_name().unwrap().to_str().unwrap()) {
                                let new_project_1 =
                                    Project::new(project.to_path_buf(), display.clone());
                                let _ = new_project_1.save();

                                let mut save_dir = match AppDirs::new(Some("Reanimator"), false) {
                                    Some(a) => {
                                        fs::create_dir_all(a.cache_dir.clone());
                                        a.cache_dir
                                    }
                                    None => current_exe().unwrap(),
                                };
                        
                                let p = save_dir
                                .join(new_project_1.name());


                                for i in fs::read_dir(p).unwrap() {
                                    if let Ok(i) = i {
                                        if i.metadata().unwrap().created().unwrap().elapsed().unwrap().as_secs()
                                        > (86400 * 7) {
                                            fs::remove_file(i.path());
                                        }
                                    }
                                }
                                new_project = Some(new_project_1);
                            }
                        }
                    });

                ui.dummy(ui.content_region_avail());
            });

        return new_project;
    }

    pub fn new(path: PathBuf, display: Display<WindowSurface>) -> Project {
        // println!("{:?}", fs::create_dir_all(path.join("nodes")));

        let mut nodes: Vec<Box<dyn MyNode>> = vec![
            NodeType::Output.new_node(),
            NodeType::DefaultImageOut.new_node(),
        ];

        nodes[0].set_xy(20.0, 150.0);
        nodes[1].set_xy(20.0, 50.0);

        let mut new = Project {
            return_to_home_menu: false,
            selected_snapshot: 0,
            storage: Storage::new(display),
            nodes,
            snapshots: vec![],
            selected: None,
            graph_offset: (0.0, 0.0),
            scale: 1.0,
            selected_input: None,
            selected_output: None,
            display_history: false,
            connections: HashMap::new(),
            metrics: false,
            path: path.clone(),
            node_edit: None,
            new_node_types: NodeType::iter()
                .map(|x| x.new_node())
                .collect::<Vec<Box<dyn MyNode>>>(),
            // node_render_target: render_target(1000, 1000),
            node_speeds: HashMap::new(),
            node_run_order: (0, vec![]),
            selected_node_to_add: NodeType::iter().len(),
        };

        new.new_node_types.sort_by(|a, b| {
            format!("{:?},{}", a.path(), a.name()).cmp(&format!("{:?},{}", b.path(), b.name()))
        });

        if let Ok(connections) =
            savefile::load_file::<HashMap<String, String>, PathBuf>(path.join("connections.bin"), 0)
        {
            new.connections = connections;

            new.nodes = vec![];

            for node_type in NodeType::iter() {
                if let Ok(node_paths) = fs::read_dir(path.join("nodes").join(node_type.name())) {
                    for node in node_paths {
                        if let Ok(node) = node {
                            if let Some(new_node) = node_type.load_node(node.path()) {
                                new.nodes.push(new_node);
                            };
                        }
                    }
                }
            }
        }

        return new;
    }

    pub fn name(&self) -> String {
        match self.path.as_path().file_name() {
            Some(a) => a.to_owned().into_string().unwrap(),
            None => format!("Name Error: {:?}", self.path),
        }
    }

    pub fn save(&self) -> Result<(), SavefileError> {
        // println!("{:?}", self.path);
        // println!("{:?}", self.path.join("connections.bin"));

        fs::create_dir_all(self.path.clone())?;

        savefile::save_file(self.path.join("connections.bin"), 0, &self.connections)?;
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

        for node in &self.nodes {

        }

    }

    pub fn render(&mut self, ui: &Ui, user_settings: &UserSettings, renderer: &mut Renderer) {
        // Context::set_pixels_per_point(&self, pixels_per_point)
        let size_array = ui.io().display_size;
        let window_size = ImVec2::new(size_array[0], size_array[1]);
        // self.node_edit = None;
        let mut time_list: Vec<f64> = vec![];
        // hash
        let connection_hash = self.nodes.len() as u64
            + calculate_hash(
                &<HashMap<String, String> as Clone>::clone(&self.connections)
                    .into_iter()
                    .collect::<Vec<(String, String)>>(),
            );
        
        
            
        
        ui.main_menu_bar(|| {
            ui.text(self.path.as_os_str().to_str().unwrap());
        });
        // renderer.render(target, draw_data)
        ui.window("nodes")
            .no_decoration()
            .bg_alpha(0.0)
            .always_auto_resize(true)
            .content_size([window_size.x + 20.0, window_size.y + 20.0])
            .movable(false)
            .position([10.0, 10.0], imgui::Condition::Appearing)
            .bring_to_front_on_focus(false)
            .build(|| {
                let mut node_pos_map: HashMap<String, ImVec2> = HashMap::new();
                // println!("{:?}", ui.mouse_drag_delta());

                if ui.is_window_focused() {
                    self.selected = None;
                }
                let mut delete_node: Option<usize> = None;
                for (i, node) in self.nodes.iter_mut().enumerate() {
                    let mut del_window_not = true;
                    

                    let mut moving = false;
                    let mut move_delta = ui.mouse_drag_delta();

                    if ui.mouse_drag_delta() != [0.0,0.0] && ui.is_window_focused() && ui.is_mouse_dragging(imgui::MouseButton::Left) {
                        moving = true;
                    } else if !ui.is_window_focused() {
                        move_delta = [0.0,0.0];
                    }

                    let mut out_of_bounds = false;

                    if node.x() + move_delta[0] > size_array[0] - 20.0
                        || node.x() + move_delta[0] < 20.0
                        || node.y() + move_delta[1] > size_array[1] - 20.0
                        || node.y() + move_delta[1] < 20.0
                    {
                        out_of_bounds = true;
                    }
                    if out_of_bounds {
                        // ui.set_window_font_scale(0.01);
                        // println!("tiny");
                        // ui.push_style_var(imgui::StyleVar::Alpha(0.0));
                    }else {
                        // ui.set_window_font_scale(1.0);
                    }
                    ui.window(format!("{}{}({})", node.name(), " ".repeat(40), node.id()))
                        .resizable(false)
                        .focus_on_appearing(true)
                        .always_auto_resize(true)
                        .opened(&mut del_window_not)
                        .bg_alpha(0.9)
                        .collapsible(false)
                        .movable(true)
                        .always_auto_resize(true)
                        // .content_size([0.0,0.0])
                        .position([node.x() + move_delta[0], node.y() + move_delta[1]], 
                            if out_of_bounds || moving {
                            imgui::Condition::Always} 
                            else {
                                imgui::Condition::Once
                            }
                        )
                        .size_constraints(
                            ui.calc_text_size(node.name() + "xxxxx"),
                            [f32::MAX, -1.0],
                        )
                        .build(|| {

                            if out_of_bounds {
                                ui.set_window_font_scale(0.8);
                                // println!("tiny");
                            }else {
                                ui.set_window_font_scale(1.0);
                            }

                            if ui.is_window_focused() {
                                self.node_edit = Some(i);
                            }

                            ui.cursor_screen_pos();

                            let window_size = ui.window_size();

                            let speed = self.node_speeds.get(&node.id());
                            // ui.set_window_font_scale(0.6);
                            if self.node_run_order.0 == connection_hash {
                                ui.text(format!(
                                    "{}. |",
                                    self.node_run_order
                                        .1
                                        .iter()
                                        .enumerate()
                                        .find_map(|(i, x)| {
                                            if **x == node.id() {
                                                Some(i.to_string())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or("NA".to_string())
                                ))
                            }
                            ui.same_line();
                            if speed.is_some() {
                                ui.text(format!(
                                    "{} micros",
                                    speed.unwrap_or(&Duration::ZERO).as_micros()
                                ));
                            } else {
                                ui.text("NA");
                            }
                            // ui.set_window_font_scale(1.0);

                            // ui.columns(2, node.id(), false);
                            for input in node.inputs() {
                                let last_pos = ui.cursor_screen_pos();
                                if ui.button(input.clone()) {
                                    self.selected_input = Some(node.input_id(input.clone()));
                                }

                                let new_pos = ui.cursor_screen_pos();
                                let average_pos = [
                                    (new_pos[0] + last_pos[0]) / 2.0 - 8.0,
                                    (new_pos[1] + last_pos[1]) / 2.0,
                                ];
                                node_pos_map
                                    .insert(node.input_id(input.clone()), average_pos.into());
                            }
                            if node.outputs().len() != 0 && node.inputs().len() != 0 {
                                ui.separator();
                            }
                            for output in node.outputs() {
                                let last_pos = ui.cursor_screen_pos();
                                if ui.button(output.clone()) {
                                    self.selected_output = Some(node.output_id(output.clone()));
                                }

                                let new_pos = ui.cursor_screen_pos();
                                let average_pos = [
                                    (new_pos[0] + last_pos[0]) / 2.0 + window_size[0] - 10.0,
                                    (new_pos[1] + last_pos[1]) / 2.0,
                                ];
                                node_pos_map
                                    .insert(node.output_id(output.clone()), average_pos.into());
                            }
                            let window_pos = ui.window_pos();
                            if !moving {
                                node.set_xy(window_pos[0], window_pos[1]);
                            }
                            if node.type_() == NodeType::Output {
                                let a: Option<&OutputNode> =
                                    (*node).as_any().downcast_ref::<OutputNode>();
                                if let Some(output_node) = a {
                                    time_list.append(&mut output_node.run_with_time.clone());
                                    if let Some(image_id) = output_node.texture_id {
                                        let pos = ui.cursor_screen_pos();
                                        if ui.button_with_size("render", [50.0, 50.0]) {
                                            time_list.push(ui.time());
                                        }
                                        let mut avail = ui.content_region_avail();
                                        avail[1] += 55.0;
                                        avail[0] += 55.0;
                                        let image_dimensions_bad = renderer
                                            .textures()
                                            .get(image_id)
                                            .unwrap()
                                            .texture
                                            .dimensions();
                                        let image_dimensions = [
                                            image_dimensions_bad.0 as f32,
                                            image_dimensions_bad.1 as f32,
                                        ];

                                        let scale = (avail[0] / image_dimensions[0])
                                            .min(avail[1] / image_dimensions[1]);
                                        ui.get_foreground_draw_list()
                                            .add_image(
                                                image_id,
                                                [pos[0], pos[1] + image_dimensions[1] * scale],
                                                [pos[0] + image_dimensions[0] * scale, pos[1]],
                                            )
                                            .build();
                                        // ui.image_button("image", image_id, [image_dimensions[0] * scale, image_dimensions[1] * scale]);
                                    }
                                }
                            }
                        });

                    if !del_window_not {
                        ui.open_popup(format!("delete node? ({})", node.id()));
                    }
                    ui.popup(format!("delete node? ({})", node.id()), || {
                        ui.text("are you sure you want to delete the node?");
                        // ui.disabled( node.type_() == NodeType::Output, || {
                        if ui.button("delete") {
                            // self.nodes.remove(node.);
                            delete_node = Some(i);
                            ui.close_current_popup();
                        }
                        // if node.type_() == NodeType::Output && ui.is_item_hovered() {
                        //     ui.tooltip_text("the output node cannot be deleted")
                        // }
                        // });
                        if ui.button("cancel") {
                            ui.close_current_popup();
                        }
                    });
                }

                // println!("{:?}", self.node_edit);

                if let Some(kill) = delete_node {
                    self.nodes.remove(kill);
                }

                let draw_list = ui.get_background_draw_list();

                let mouse_pos = ui.io().mouse_pos;
                match (
                    self.selected_input.clone(),
                    self.selected_output.clone(),
                    mouse_pos,
                ) {
                    (None, Some(a), m) => {
                        if let Some(pos) = node_pos_map.get(&a) {
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
                        if let Some(pos) = node_pos_map.get(&a) {
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
                    if let Some(pos2) = node_pos_map.get(a) {
                        if let Some(pos) = node_pos_map.get(b) {
                            draw_list
                                .add_bezier_curve(
                                    [pos.x, pos.y],
                                    [(pos.x + pos2.x) * 0.5, pos.y],
                                    [(pos.x + pos2.x) * 0.5, pos2.y],
                                    [pos2.x, pos2.y],
                                    ImColor32::BLACK,
                                )
                                .build();
                        }
                    }
                }
            });

        for t in time_list {
            self.storage.time = t;
            self.run_nodes(renderer);
        }

        let mut left_sidebar_width = 0.0;

        ui.window("sidebar")
            .no_decoration()
            .position(
                [0.0, ui.calc_text_size("x")[1] * 1.5],
                imgui::Condition::Always,
            )
            .size_constraints([0.0, window_size.y], [window_size.x * 0.4, window_size.y])
            .resizable(true)
            // .always_auto_resize(true)
            .build(|| {
                // Style::use_light_colors(&mut self)
                if ui.button("Add Node") {
                    ui.open_popup("Add Node");
                }
                self.new_node_menu(ui);

                if ui.button("debug") {
                    self.metrics = !self.metrics;
                }

                if ui.button("save") {
                    println!("save button, {:?}", self.save());
                }

                if ui.button("return home") {
                    self.return_to_home_menu = true;
                }

                if ui.button("timeline") {
                    self.display_history = !self.display_history;
                }

                if self.display_history {
                    self.history_window(ui);
                }

                left_sidebar_width = ui.window_size()[0];
            });

        ui.window("edit_node")
            .collapsible(true)
            .position_pivot([0.0, 1.0])
            .position(
                [left_sidebar_width, size_array[1]],
                imgui::Condition::Always,
            )
            .size_constraints(
                [size_array[0] - left_sidebar_width, 0.0],
                [size_array[0] - left_sidebar_width, size_array[1] * 0.5],
            )
            .build(|| {
                // println!("{:?}", self.node_edit);
                match self.node_edit {
                    Some(a) if self.nodes.len() > a => {
                        self.nodes[a].edit_menu_render(ui, renderer);
                    }
                    _ => ui.text("no node has been selected"),
                }
            });

        if self.metrics {
            ui.show_metrics_window(&mut self.metrics);
        }

        // for i in self.nodes {

        // self.run_nodes();

        // }

        // ui.show_demo_window(&mut true);

        // ui.show_user_guide();
    }

    fn run_nodes(&mut self, renderer: &mut Renderer) {
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
                if n.type_() == NodeType::Output {
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
                    }
                }
            }
        }
    }

    fn new_node_menu(&mut self, ui: &Ui) {
        let mut group: HashMap<String, Option<TreeNodeToken>> = HashMap::new();

        ui.modal_popup_config("Add Node").build(|| {
            ui.columns(2, "select new node col", true);
            for n in 0..self.new_node_types.len() {
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
                    ui.radio_button(
                        self.new_node_types[n].name(),
                        &mut self.selected_node_to_add,
                        n,
                    );
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
                    new_node2.set_xy(150.0, 150.0);
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
                self.new_node_types[self.selected_node_to_add].description(ui);
            } else {
                ui.text("no node has been selected");
            }

            // self.new_node_types[n].;
        });
    }
}
