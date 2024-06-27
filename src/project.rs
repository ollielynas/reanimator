use imgui::{
    color, draw_list, internal::RawCast, sys::{igGetWindowSize, ImColor, ImVec2}, ImColor32, Image, Style, Ui, WindowFlags
};use glium::{program, Display, Program, Surface};

use savefile::SavefileError;
use strum::IntoEnumIterator;
use std::{collections::HashMap, fs, path::PathBuf, time::{Duration, Instant}};
use glium::{backend::Facade, glutin::{surface::WindowSurface},  Texture2d};

use crate::{
    node::MyNode,
    nodes::{self, image_io::*}, storage::Storage, user_info::{self, UserSettings},
};
use crate::nodes::node_enum::*;
use rfd::FileDialog;



pub struct Project {
    pub storage: Storage,
    pub nodes: Vec<Box<dyn MyNode>>,
    pub time: f32,
    pub selected: Option<usize>,
    graph_offset: (f32, f32),
    scale: f32,
    selected_input: Option<String>,
    selected_output: Option<String>,
    connections: HashMap<String, String>,
    metrics: bool,
    add_node_window: bool,
    new_node_types: Vec<Box<dyn MyNode>>,
    pub path: PathBuf,
    pub node_speeds: HashMap<String, Duration>,
}

impl Project {
    pub fn project_menu(ui: &mut Ui, display: &Display<WindowSurface>, user_settings: &mut UserSettings) -> Option<Project> {
        let mut new_project = None;
        let size_array = ui.io().display_size;

        let height = size_array[1] * 0.8;
        let pos = [size_array[0] * 0.5 -height * 0.5, size_array[1] * 0.1];
        // let size = ui.siz
        ui.window("project selector")
        .collapsible(false)
        .movable(false)
        .bring_to_front_on_focus(false)
        .always_auto_resize(true)
        .size([height, height], imgui::Condition::Always)
        .position(pos, imgui::Condition::Always).build(|| {
            ui.input_text("##", &mut user_settings.new_project_name)
            .auto_select_all(true)
            .build();
            if ui.button("create new project") {
                new_project = Some(Project::new(user_settings.project_folder_path.join(user_settings.new_project_name.clone()), display.clone()));
            }
            ui.separator();
            // ui.begin_group();
            // ui.
            // ui.indent();
            ui.set_window_font_scale(1.2);
            ui.text("load project");
            ui.set_window_font_scale(1.0);
            if ui.small_button("change project folder") {
                let new_folder = FileDialog::new().set_directory(&user_settings.project_folder_path)
                .set_can_create_directories(true)
                .set_title("set project folder")
                .pick_folder();

                if new_folder.is_some() {
                    user_settings.project_folder_path = new_folder.unwrap();
                    user_settings.update_projects();
                    user_settings.save();
                }
            }

        
            ui.window("projects")
            .always_vertical_scrollbar(true)
            .position_pivot([0.5,1.0])
            // .focused(true)
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .position([size_array[0]*0.5, size_array[1] - height * 0.15], imgui::Condition::Always)
            .size([height * 0.9, height - ui.cursor_pos()[1] - 20.0] , imgui::Condition::Always)
            .build(|| {
            for project in &user_settings.projects {
                if ui.button(project.file_name().unwrap().to_str().unwrap()) {
                    new_project = Some(Project::new(project.to_path_buf(), display.clone()));
                }
            }});


            
            
            
            


        
        });
        return new_project;
    }


    fn new(path: PathBuf, display: Display<WindowSurface>) -> Project {

        println!("{:?}",fs::create_dir_all(path.join("nodes")));


        let mut new = Project {
            add_node_window: false,
            storage: Storage::new(display),
            nodes: vec![NodeType::Output.new_node(), NodeType::DefaultImageOut.new_node()],
            time: 0.0,
            selected: None,
            graph_offset: (0.0, 0.0),
            scale: 1.0,
            selected_input: None,
            selected_output: None,
            connections: HashMap::new(),
            metrics: false,
            path: path.clone(),
            new_node_types: NodeType::iter().map(|x| x.new_node()).collect::<Vec<Box<dyn MyNode>>>(),
            // node_render_target: render_target(1000, 1000),
            node_speeds: HashMap::new(),
        };
        new.new_node_types.sort_by(|a,b| format!("{:?},{}",a.path(),a.name()).cmp(&format!("{:?},{}",b.path(),b.name())));



        if let Ok(connections)  = savefile::load_file::<HashMap<String, String>, PathBuf>(path.join("connections.bin"), 0) {
            new.connections = connections;

            new.nodes = vec![];

            for node_type in NodeType::iter() {
                if let Ok(node_paths) = fs::read_dir(path.join("nodes").join(node_type.name())) {
                    for node in node_paths {
                        if let Ok(node) = node {
                            let name = node.path().file_name().unwrap().to_str().unwrap().to_owned().replace(".bin", "");
                            if let Some(new_node) = node_type.load_node(name, node.path()) {
                                new.nodes.push(new_node);
                            };
                        }
                    }
                }
            }
        }
        
        return new;
    }

    fn save(&self) -> Result<(), SavefileError> {
        
        println!("{:?}", self.path);
        println!("{:?}", self.path.join("connections.bin"));

        

        savefile::save_file(self.path.join("connections.bin"), 0, &self.connections)?;
        fs::remove_dir_all(self.path.join("nodes"))?;
        fs::create_dir_all(self.path.join("nodes"))?;
        for node in &self.nodes {
            fs::create_dir_all(self.path.join("nodes").join(node.name()))?;
            node.save(self.path.join("nodes"))?;
        }

        return Ok(());

    }

    pub fn render(&mut self, ui: &mut Ui, user_settings: &UserSettings) {
        // Context::set_pixels_per_point(&self, pixels_per_point)
        let size_array = ui.io().display_size;
        let window_size = ImVec2::new(size_array[0], size_array[1]);

        ui.window("nodes")
            .no_decoration()
            .bg_alpha(0.0)
            .always_auto_resize(true)
            .content_size([window_size.x, window_size.y])
            .movable(true)
            .position([0.0, 0.0], imgui::Condition::Appearing)
            .bring_to_front_on_focus(false)
            .build(|| {
                let mut node_pos_map: HashMap<String, ImVec2> = HashMap::new();

                if ui.is_window_focused() {
                    self.selected = None;
                }
                let mut delete_node: Option<usize> = None;
                for (i, node) in self.nodes.iter_mut().enumerate() {
                    let mut del_window_not = true;
                    ui.window(format!("{}{}({})",node.name()," ".repeat(60),node.id()))
                        .resizable(false)
                        .focus_on_appearing(true)
                        .always_auto_resize(true)
                        .opened(&mut del_window_not)
                        .bg_alpha(0.9)
                        .collapsible(false)
                        .movable(true)
                        .always_auto_resize(true)
                        .content_size([0.0,0.0])
                        .position([node.x(),node.y()], imgui::Condition::Once)
                        .size_constraints(ui.calc_text_size(node.name()), [-1.0,-1.0])
                        .build(|| {
                            
                            let window_size = ui.window_size();
                            
                            if let Some(speed) = self.node_speeds.get(&node.id()) {
                                ui.set_window_font_scale(0.6);
                                ui.text(format!("{} micros", speed.as_micros()));
                                ui.set_window_font_scale(1.0);
                            }

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
                            // ui.new_line();
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
                            node.set_xy(window_pos[0], window_pos[1]);
                        });

                        
                        if !del_window_not {
                            ui.open_popup(format!("delete node? ({})",node.id()));
                        }
                        ui.popup(format!("delete node? ({})",node.id()), || {
                            ui.text("are you sure you want to delete the node?");
                            if ui.button("delete") {
                                // self.nodes.remove(node.);
                                delete_node = Some(i);
                                ui.close_current_popup();
                            }
                            if ui.button("cancel") {
                                ui.close_current_popup();
                            }
                        });

                }

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
                            draw_list
                                .add_bezier_curve(
                                    m,
                                    [(m[0] + pos.x)*0.5, m[1]],
                                    [(m[0] + pos.x)*0.5, pos.y],
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
                                    [(m[0] + pos.x)*0.5, pos.y],
                                    [(m[0] + pos.x)*0.5, m[1]],
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
                                [(pos.x + pos2.x)*0.5, pos.y],
                                [(pos.x + pos2.x)*0.5, pos2.y],
                                [pos2.x, pos2.y],
                                ImColor32::BLACK,
                            )
                            .build();
                    
                }
            }
        }
    });

        ui.window("sidebar")
            .movable(false)
            .resizable(true)
            .no_decoration()
            .position([0.0, 0.0], imgui::Condition::Appearing)
            .size_constraints([0.0,window_size.y], [-1.0,window_size.y])
            .always_auto_resize(true)
            .build(|| {
                // Style::use_light_colors(&mut self)
                if ui.button("add node") {
                    self.add_node_window = !self.add_node_window;
                }
                if ui.button("debug") {
                    self.metrics = !self.metrics;
                }

                if ui.button("run all") {
                    self.run_nodes();
                }
                if ui.button("save") {
                    println!("save button, {:?}",self.save());
                }

                

            });

        if self.metrics {
            ui.show_metrics_window(&mut self.metrics);
        }
        if self.add_node_window {
            self.new_node_menu(ui);
        }

        // ui.show_user_guide();
    }

    fn run_nodes(&mut self) {
        self.node_speeds.clear();
        for i in &self.nodes {
            let now = Instant::now();
            i.run(&mut self.storage, self.connections.clone());
            self.node_speeds.insert(i.id(), now.elapsed());
        }
    }

    fn new_node_menu(&mut self, ui: &mut Ui) {
        let mut group: Vec<&str> = vec![];
        ui.window("New Node")
        .build(|| {
            for n in &self.new_node_types {
                if n.path() != group {
                    ui.text(n.path().join("/"));
                }
                if ui.small_button(n.name()) {
                    self.nodes.push(n.type_().new_node())
                }
            }
        });
    }
}
