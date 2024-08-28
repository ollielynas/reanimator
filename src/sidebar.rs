use std::{ffi::OsString, fs};

use imgui::{sys::ImVec2, Ui};
use rfd::FileDialog;

use crate::{
project::Project, render_nodes::RenderNodesParams,
    user_info::UserSettings,
};

pub struct SidebarParams {
    pub new_node_popup: bool,
    pub left_sidebar_width: f32,
    pub menu_bar_size: [f32; 2],
}

impl Default for SidebarParams {
    fn default() -> Self {
        SidebarParams {
            new_node_popup: false,
            left_sidebar_width: 0.0,
            menu_bar_size: [0.0, 0.0],
        }
    }
}


impl Project {
    pub fn render_sidebar(
        &mut self,
        params: &mut RenderNodesParams,
        ui: &Ui,
        sidebar_params: &mut SidebarParams,
        user_settings: &mut UserSettings,
    ) {
        let size_array = ui.io().display_size;
        let window_size = ImVec2::new(size_array[0], size_array[1]);

        ui.window("sidebar")
            .no_decoration()
            .focused(self.recenter)
            .position(
                [0.0, sidebar_params.menu_bar_size[1] - 1.0],
                imgui::Condition::Always,
            )
            .size_constraints([0.0, window_size.y], [window_size.x * 0.4, window_size.y])
            .resizable(true)
            .collapsible(false)
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

                match self.edit_tab {
                    crate::generic_io::EditTab::Nodes => {
                        // Style::use_light_colors(&mut self)
                        if ui.button("add node") || sidebar_params.new_node_popup {
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
                        ui.separator();

                        ui.checkbox("auto update", &mut self.project_settings.render_ticker);
                    }
                    crate::generic_io::EditTab::BatchFileEdit => {
                        ui.set_window_font_scale(1.3);
                        if ui.button("run batch") {
                            self.project_settings.batch_files.run = true;
                        }
                        ui.set_window_font_scale(1.0);
                    }
                    crate::generic_io::EditTab::ProjectRes => {
                        
                        if ui.button("reload files") {
                            self.project_settings.local_files.reload(&self.storage);
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

                if ui.button("export project") {
                    self.export();
                }
                if ui.button("return home") {
                    self.save();
                    self.return_to_home_menu = true;
                }
                if self.display_history {
                    self.history_window(ui);
                }

                sidebar_params.left_sidebar_width = ui.window_size()[0];
                for i in sidebar_col_things {
                    i.end();
                }
                for i in sidebar_things {
                    i.end();
                }

                if self.new_node_menu(ui,  &user_settings) {
                    params.moving = false;
                    params.scale_changed = false;
                }

                if ui.is_window_hovered() {
                    params.scale_changed = false;
                }
            });
    }
}
