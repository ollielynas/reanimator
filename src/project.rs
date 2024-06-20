use imgui::{
    draw_list, internal::RawCast, sys::{igGetWindowSize, ImColor, ImVec2}, ImColor32, Image, Style, Ui, WindowFlags
};
use strum::IntoEnumIterator;
use std::collections::HashMap;
use glium::{backend::Facade, program, Program};

use crate::{
    node::{MyNode},
    nodes::image_io::*,
};
use crate::nodes::node_enum::*;


pub struct Storage {
    frames: HashMap<String, Image>,
}

impl Storage {
    fn new() -> Storage {

        Storage {
            frames: HashMap::new(),
        }
    }


    pub fn set_frame(&mut self, k:String, v: Image) {
        self.frames.insert(k, v);
    }

    pub fn get_frame(&self, k:&String) -> Option<Image> {
        // do something about this copy, very wasteful
        self.frames.get(k).copied()
    }
}

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

    // node_render_target: RenderTarget,
}

impl Project {
    pub fn project_menu(ui: &mut Ui) -> Option<Project> {
        ui.window_pos();
        if ui.button("new project") {
            return Some(Project::new("./debug".to_owned()));
        }
        return None;
    }

    fn new(path: String) -> Project {
        let mut new = Project {
            add_node_window: false,
            storage: Storage::new(),
            nodes: vec![NodeType::Output.new_node(), NodeType::Debug.new_node()],
            time: 0.0,
            selected: None,
            graph_offset: (0.0, 0.0),
            scale: 1.0,
            selected_input: None,
            selected_output: None,
            connections: HashMap::new(),
            metrics: false,
            new_node_types: NodeType::iter().map(|x| x.new_node()).collect::<Vec<Box<dyn MyNode>>>()
            // node_render_target: render_target(1000, 1000),
        };
        new.new_node_types.sort_by(|a,b| format!("{:?},{}",a.path(),a.name()).cmp(&format!("{:?},{}",b.path(),b.name())));
        return new;
    }

    pub fn render(&mut self, ui: &mut Ui) {
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
                for node in self.nodes.iter_mut() {
                    ui.window(format!("{}{}({})",node.name()," ".repeat(40),node.id()))
                        .resizable(false)
                        .focus_on_appearing(true)
                        .always_auto_resize(true)
                        .bg_alpha(0.9)
                        .collapsible(false)
                        .movable(true)
                        .always_auto_resize(false)
                        .content_size([0.0,0.0])
                        .build(|| {
                            // ui.columns(2, node.id(), false);
                            let window_size = ui.window_size();
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
            .always_auto_resize(true)
            .build(|| {
                // Style::use_light_colors(&mut self)
                if ui.button("add node") {
                    self.add_node_window = !self.add_node_window;
                }
                if ui.button("debug") {
                    self.metrics = !self.metrics;
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
