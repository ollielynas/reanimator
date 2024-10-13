

use imgui::Ui;
use textdistance::{self, Algorithm, Cosine};
use win_screenshot::prelude::capture_display;
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

pub struct AdvancedColorPicker {
    pub color: [f32; 4],
    pub open: bool,
    color_picker_mode: imgui::ColorPickerMode,
    pantone_colors: Vec<PantoneColor>,
    search_name: String,
    search_code: String,
    search_catagories: Vec<String>,
    // search_results: Vec<PantoneColor>,
    result_count: i32,
}

struct PantoneColor {
    color: [f32; 4],
    name: String,
    code: String,
    category: String,
}

impl PantoneColor {
    fn load_colors(only_named: bool) -> Vec<PantoneColor> {
        let mut col = vec![];

        let mut name: Option<String> = None;
        let mut category: Option<String> = None;
        let mut code: Option<String> = None;
        let mut color: Option<[f32; 4]> = None;

        for line in include_str!("set1.txt").lines() {
            if line == "" {
                continue;
            }

            if line.starts_with("code") {
                code = Some(line.replace("code: ", "").replace(",", ""))
            }
            if line.starts_with("name") {
                name = Some(line.replace("name: ", "").replace(",", ""));
            }
            if line.starts_with("category") {
                category = Some(line.replace("category: ", "").replace(",", ""))
            }
            if line.starts_with("rgb") {
                let mut line2 = line.to_string();
                line2 = line2.replace("rgb: ", "");
                let nums = line2.split(",").collect::<Vec<&str>>();
                if let (Ok(r), Ok(g), Ok(b)) = (
                    nums[0].parse::<u8>(),
                    nums[1].parse::<u8>(),
                    nums[2].parse::<u8>(),
                ) {
                    color = Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]);
                }
            }

            let mut all_some = false;
            if let (Some(name), Some(code), Some(category), Some(color)) =
                (&name, &code, &category, &color)
            {
                col.push(PantoneColor {
                    name: name.to_string(),
                    color: *color,
                    code: code.to_string(),
                    category: category.to_string(),
                });
                all_some = true;
            }
            if all_some {
                name = None;
                code = None;
                category = None;
                color = None;
            }
        }

        if only_named {
            col.retain(|x| x.name != "");
        }

        return col;
    }
}

impl Default for AdvancedColorPicker {
    fn default() -> Self {
        Self {
            color: [1.0; 4],
            open: false,
            pantone_colors: vec![],
            color_picker_mode: imgui::ColorPickerMode::HueBar,
            search_name: "".to_owned(),
            search_code: "".to_owned(),
            search_catagories: vec![],
            result_count: 10,
        }
    }
}

impl AdvancedColorPicker {
    pub fn search_color(&mut self) {
        self.pantone_colors.sort_by(|a, b| {
            ((a.color[0] - self.color[0]).abs()
                + (a.color[1] - self.color[1]).abs()
                + (a.color[2] - self.color[2]).abs())
            .total_cmp(
                &((b.color[0] - self.color[0]).abs()
                    + (b.color[1] - self.color[1]).abs()
                    + (b.color[2] - self.color[2]).abs()),
            )
        });
    }
    pub fn search_text(&mut self) {
        let alg = Cosine::default();

        self.pantone_colors.sort_by_cached_key(|x| {
            ((alg.for_str(&x.code, &self.search_code).ndist()
                + alg.for_str(&x.name, &self.search_name).ndist())
                * 100000.0) as i32
        });
    }

    pub fn render(&mut self, ui: &Ui) {
        if !self.open {
            return;
        }
        let mut open = self.open;
        ui.window("Advanced Color Picker")
            .opened(&mut open)
            .build(|| {
                if let Some(_tab_bar) = ui.tab_bar("color tab bar") {
                    if let Some(_tab_iter) = ui.tab_item("Color Picker") {
                        // ui.columns(2, "advanced col picker", true);
                        ui.set_next_item_width(
                            0.6 * ui.content_region_avail()[0].min(ui.content_region_avail()[1]),
                        );
                        ui.color_picker4_config("color_picker", &mut self.color)
                            .alpha(true)
                            .display_hex(true)
                            .display_hsv(true)
                            .display_rgb(true)
                            .mode(self.color_picker_mode)
                            .build();
                        ui.next_column();
                        if ui.button(format!(
                            "Switch to {}",
                            match self.color_picker_mode {
                                imgui::ColorPickerMode::HueBar => "Wheel",
                                imgui::ColorPickerMode::HueWheel => "Bar",
                            }
                        )) {
                            self.color_picker_mode = match self.color_picker_mode {
                                imgui::ColorPickerMode::HueBar => imgui::ColorPickerMode::HueWheel,
                                imgui::ColorPickerMode::HueWheel => imgui::ColorPickerMode::HueBar,
                            }
                        }
                    }

                    if let Some(_tab_iter) = ui.tab_item("Pick Color") {
                        let mut color = self.get_cursor_color(ui);
                        ui.color_edit4("old color", &mut self.color);
                        ui.color_edit4("selected color", &mut color);
                        ui.text("press shift to select color");
                        if ui.io().key_shift {
                            self.color = color;
                        }
                    }
                    if let Some(_tab_iter) = ui.tab_item("Pantone Colors") {
                        // ui.columns(2, "pantone search", true);
                        ui.text(format!("loaded {} colors", self.pantone_colors.len()));

                        if ui.button("load colors") {
                            self.pantone_colors = PantoneColor::load_colors(false);
                        }
                        ui.same_line();
                        if ui.button("load only named colors") {
                            self.pantone_colors = PantoneColor::load_colors(true);
                        }

                        if ui.button("find similar") {
                            self.search_color();
                        };

                        ui.spacing();
                        ui.spacing();

                        ui.text("Name:");
                        ui.input_text("name", &mut self.search_name).build();

                        ui.text("Code:");
                        ui.input_text("code", &mut self.search_code).build();
                        if ui.button("Search") {
                            self.search_text();
                        }

                        ui.next_column();

                        ui.input_int("result count", &mut self.result_count).build();

                        self.result_count = self.result_count.clamp(0, 50);

                        for i in 0..(self.result_count as usize).min(self.pantone_colors.len()) {
                            let color = &self.pantone_colors[i];
                            let a = ui.color_button(&color.code, color.color);
                            ui.same_line();


                            let (name, code) = (&color.name, &color.code);
                            let b = ui.button(if (&color.name).is_empty() {
                                code
                            } else {
                                name
                            });

                            if a || b {
                                self.color = color.color;
                            }
                        }
                    }
                }
            });
        self.open = open;
    }

    fn get_cursor_color(&mut self, _ui: &Ui) -> [f32; 4] {
        let mut color = [0.0; 4];
        if let Ok(display) = capture_display() {
            let mut global_pos: POINT = POINT::default();
            unsafe { GetCursorPos(&mut global_pos) };
            let index = (display.width as i32 * global_pos.y + global_pos.x) * 4;
            if index >= 0 && index < display.pixels.len() as i32 - 2 {
                let r = display.pixels[index as usize];
                let g = display.pixels[index as usize + 1];
                let b = display.pixels[index as usize + 2];
                color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
            }
        }
        return color;
    }
}
