use imgui::{Ui, Window};
use lumo::tracer::Color;




/// link can be a file path pr a url
pub fn link_widget(ui: &Ui, text: String, link: String) {
    ui.same_line();
    // let pos = ui.cursor_pos();
    ui.text_colored([0.0, 102.0/255.0, 204.0/255.0, 1.0], text);

    

    if ui.is_item_hovered() {
        let p1 = ui.item_rect_max();
        let p2 = [ui.item_rect_min()[0], p1[1]];
        ui.get_window_draw_list().add_line(p1, p2, [0.0, 102.0/255.0, 204.0/255.0, 1.0]).build();
        if ui.is_mouse_clicked(imgui::MouseButton::Left) {
            open::that(&link);
        }
        ui.tooltip_text(link);
    }


}