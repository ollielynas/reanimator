use glium::{program, BlitTarget, Display, Program, Surface};
use imgui::drag_drop::PayloadIsWrongType;
use imgui::{sys::ImVec2, ImColor32, TreeNodeToken, Ui};
use imgui::{DrawListMut, Style, WindowHoveredFlags};
use imgui_glium_renderer::Renderer;
use platform_dirs::AppDirs;
use std::collections::HashSet;
use std::thread::sleep;
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
use crate::nodes::output::image_io::OutputNode;
use crate::project::{graph_to_screen_pos, screen_to_graph_pos, Project};
use crate::{
    advanced_color_picker::AdvancedColorPicker, history_tracker::Snapshot, node,
    nodes::node_enum::*,
};
use crate::{
    node::MyNode,
    storage::Storage,
    user_info::{self, UserSettings},
};
use rfd::FileDialog;

pub struct RenderNodesParams {
    pub duplicate_node: Option<Box<dyn MyNode>>,
    pub move_delta: [f32; 2],
    pub size_array: [f32; 2],
    pub moving: bool,
    pub connection_hash: u64,
    pub scale_changed: bool,
    pub node_pos_map: HashMap<String, ImVec2>,
    pub time_list: Vec<f64>,
    pub delete_node: Option<usize>,
}

impl Project {
    pub fn render_node(
        &mut self,
        ui: &Ui,
        params: &mut RenderNodesParams,
        renderer: &mut Renderer,
    ) {
        // let mut focus

        for (i, node) in self.nodes.iter_mut().enumerate().rev() {
            let mut del_window_not = true;

            let node_screen_pos =
                graph_to_screen_pos([node.x(), node.y()], self.graph_offset, self.scale);

            let mut node_window_size = [0.0, 0.0];
            let mut node_window_pos = [0.0, 0.0];

            ui.window(format!(
                "{}{}##({})",
                node.name(),
                " ".repeat(40),
                node.id()
            ))
            .resizable(false)
            .focus_on_appearing(true)
            .opened(&mut del_window_not)
            .scroll_bar(false)
            // .focused()
            .bg_alpha(0.9)
            .bring_to_front_on_focus(false)
            .collapsible(false)
            .movable(true)
            .always_auto_resize(true)
            .position(
                [node_screen_pos[0], node_screen_pos[1]],
                imgui::Condition::Always,
            )
            .size_constraints(
                [
                    ui.calc_text_size(node.name() + "xxxxx")[0] * self.scale,
                    ui.calc_text_size(node.name() + "xxxxx")[1] * self.scale,
                ],
                [f32::MAX, -1.0],
            )
            .build(|| {
                // let _ = ui.begin_disabled(out_of_bounds);
                node_window_pos = ui.window_pos();
                // log::info!("{:?}", move_delta);
                let mut move_this_node = false;
                if ui.is_window_hovered() && params.moving {
                    params.moving = false;
                    move_this_node = true;
                    ui.set_mouse_cursor(Some(imgui::MouseCursor::Hand))
                }

                if ui.is_window_hovered() && ui.is_window_focused() && params.moving {
                    params.moving = false;
                }

                ui.set_window_font_scale(self.scale);

                if self.project_settings.generic_io.input_id == Some(node.id()) {
                    ui.text("generic input")
                }
                if self.project_settings.generic_io.output_id == Some(node.id()) {
                    ui.text("generic output")
                }

                if ui.is_window_focused() {
                    self.node_edit = Some(i);
                }

                // ui.cursor_screen_pos();

                let window_size = ui.window_size();

                // ui.columns(2, node.id(), false);
                for input in node.inputs() {
                    let last_pos = ui.cursor_screen_pos();
                    if ui.button(input.clone()) {
                        self.selected_input = Some(node.input_id(input.clone()));
                    }

                    let new_pos = ui.cursor_screen_pos();
                    let average_pos = [
                        (new_pos[0] + last_pos[0]) / 2.0 - 8.0 * self.scale,
                        (new_pos[1] + last_pos[1]) / 2.0,
                    ];
                    params
                        .node_pos_map
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
                        (new_pos[0] + last_pos[0]) / 2.0 + window_size[0] - 10.0 * self.scale,
                        (new_pos[1] + last_pos[1]) / 2.0,
                    ];
                    params
                        .node_pos_map
                        .insert(node.output_id(output.clone()), average_pos.into());
                }
                let mut window_pos = ui.window_pos();

                if move_this_node && !params.scale_changed {
                    // log::info!("moving");
                    window_pos[0] += params.move_delta[0];
                    window_pos[1] += params.move_delta[1];
                }
                let window_pos_relative_to_graph =
                    screen_to_graph_pos(window_pos, self.graph_offset, self.scale);
                if move_this_node {
                    node.set_xy(
                        window_pos_relative_to_graph[0],
                        window_pos_relative_to_graph[1],
                    );
                }
                if node.type_() == NodeType::Output {
                    let a: Option<&OutputNode> = (*node).as_any().downcast_ref::<OutputNode>();
                    if let Some(output_node) = a {
                        params
                            .time_list
                            .append(&mut output_node.run_with_time.clone());
                        if let Some(image_id) = output_node.texture_id {
                            let avail = [50.0 * self.scale, 50.0 * self.scale];
                            let image_dimensions_bad = renderer
                                .textures()
                                .get(image_id)
                                .unwrap()
                                .texture
                                .dimensions();
                            let image_dimensions =
                                [image_dimensions_bad.0 as f32, image_dimensions_bad.1 as f32];

                            let scale = (avail[0] / image_dimensions[0])
                                .min(avail[1] / image_dimensions[1]);
                            // ui.get_foreground_draw_list()
                            //     .add_image(
                            //         image_id,
                            //         [pos[0], pos[1] + image_dimensions[1] * scale],
                            //         [pos[0] + image_dimensions[0] * scale, pos[1]],
                            //     )
                            //     .build();
                            if scale != 0.0
                                && image_dimensions[0] != 0.0
                                && image_dimensions[1] != 0.0
                            {
                                if ui.image_button(
                                    "image",
                                    image_id,
                                    [image_dimensions[0] * scale, image_dimensions[1] * scale],
                                ) {
                                    params.time_list.push(ui.time());
                                }
                            }
                        }
                    }
                }

                if let Some(popup) = ui.begin_popup_context_window() {
                    if node.inputs().len() == 0 && node.outputs().len() == 1 {
                        if ui.menu_item("set as generic input") {
                            self.project_settings.generic_io.input_id = Some(node.id());
                        }
                    }
                    if node.inputs().len() == 1 && node.outputs().len() == 0 {
                        if ui.menu_item("set as generic output") {
                            self.project_settings.generic_io.output_id = Some(node.id());
                        }
                    }

                    if ui.menu_item("duplicate") {
                        let a = fs::create_dir_all(self.path.join("temp").join(node.name()));
                        if let Err(e) = a {
                            log::info!("{e:?}")
                        }
                        node.save(self.path.join("temp"));
                        let node_clone = node.type_().load_node(
                            self.path
                                .join("temp")
                                .join(node.name())
                                .join(node.id() + ".bin"),
                        );
                        if let Some(n) = node_clone {
                            // n.run(&mut Storage::new(r), self.connections, renderer);
                            params.duplicate_node = Some(n);
                        }
                        fs::remove_dir_all(self.path.join("temp"));
                    }
                    if ui.menu_item("pop editor window") {
                        self.pop_out_edit_window.insert(node.id(), true);
                    }
                }

                node_window_size = ui.window_size();
            }); // end of node window

            let mut focus_pop_out_window = false;
            if let Some(open) = self.pop_out_edit_window.get_mut(&node.id()) {
                if *open {
                    let node_window_vars2 = [
                        ui.push_style_var(imgui::StyleVar::ItemSpacing([3.0, 3.0])),
                        ui.push_style_var(imgui::StyleVar::WindowPadding([10.0, 10.0])),
                        ui.push_style_var(imgui::StyleVar::FramePadding([5.0, 5.0])),
                        ui.push_style_var(imgui::StyleVar::WindowMinSize([5.0, 5.0])),
                    ];
                    ui.window(format!("edit {} ({})", node.name(), node.id()))
                        .position(ui.io().mouse_pos, imgui::Condition::Appearing)
                        .opened(open)
                        .build(|| {
                            if ui.is_window_hovered() && ui.is_mouse_down(imgui::MouseButton::Left)
                            {
                                params.moving = false;
                            }

                            node.edit_menu_render(ui, renderer, &self.storage);
                            if ui.is_window_focused() || ui.is_any_item_hovered() {
                                focus_pop_out_window = true;
                            }
                        });
                }
            }

            if Some(i) == self.node_edit || focus_pop_out_window {
                // let w_pos = ui.window_pos();
                ui.get_background_draw_list()
                    .add_rect(
                        [node_window_pos[0], node_window_pos[1]],
                        [
                            node_window_pos[0] + node_window_size[0],
                            node_window_pos[1] + node_window_size[1],
                        ],
                        if focus_pop_out_window {
                            ImColor32::from_rgb(80, 200, 80)
                        } else {
                            ImColor32::from_rgb(200, 80, 80)
                        },
                    )
                    .rounding(2.5)
                    .thickness(3.0)
                    // .filled(true)
                    .build();

                // log::info!("x");
            }

            if !del_window_not {
                ui.open_popup(format!("delete node? ({})", node.id()));
            }
            ui.popup(format!("delete node? ({})", node.id()), || {
                ui.text("are you sure you want to delete the node?");
                // ui.disabled( node.type_() == NodeType::Output, || {
                if ui.button("delete") {
                    // self.nodes.remove(node.);
                    params.delete_node = Some(i);
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
        } // end of node loop
    }

    pub fn render_background(&self, ui: &Ui, bg_draw_list: &DrawListMut) {
        let size_array = ui.io().display_size;

        let mut gap = 50.0 * self.scale;

        while gap < 10.0 {
            gap *= 5.0;
        }

        let mut x = -self.graph_offset[0] * self.scale % gap - gap;

        while x < size_array[0] {
            let mut y = -self.graph_offset[1] * self.scale % gap - gap;
            while y < size_array[1] {
                y += gap;
                bg_draw_list
                    .add_circle(
                        [x, y],
                        (2.0 * gap / 80.0).clamp(0.5, 1.0),
                        [0.3, 0.3, 0.3, 1.0],
                    )
                    .filled(true)
                    .build();
            }
            x += gap;
        }
    }
}
