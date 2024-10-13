use std::{path::PathBuf, str::FromStr};

use imgui::{Ui};


/// link can be a file path pr a url
pub fn link_widget(ui: &Ui, text: impl Into<String>, link: impl Into<String> + Clone) {
    ui.same_line();
    // let pos = ui.cursor_pos();
    ui.text_colored([0.0, 102.0 / 255.0, 204.0 / 255.0, 1.0], text.into());

    if ui.is_item_hovered() {
        let p1 = ui.item_rect_max();
        let p2 = [ui.item_rect_min()[0], p1[1]];
        ui.get_window_draw_list()
            .add_line(p1, p2, [0.0, 102.0 / 255.0, 204.0 / 255.0, 1.0])
            .build();
        if ui.is_mouse_clicked(imgui::MouseButton::Left) {
            let _ = open::that(&(link.into()));
        }else {
            ui.tooltip_text(&(link.into()));
    }
    }
}

pub fn path_input(ui: &Ui, text: impl Into<String>, path: &mut PathBuf) {
    
    let mut input_buffer = path.display().to_string();
    ui.input_text(text.into(), &mut input_buffer).build();
    if let Ok(new_path) = PathBuf::from_str(&input_buffer) {
        *path = new_path;
    }
}
