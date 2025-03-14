// use win_desktop_duplication::*;
// use win_desktop_duplication::{tex_reader::*, devices::*};

// use rusty_duplication::{
//     capturer::{model::Capturer, simple::SimpleCapturer},
//     manager::Manager,
//     utils::{FrameInfoExt, OutputDescExt},
// };


use regex::Regex;

use std::{any::Any, collections::HashMap, path::PathBuf};
use win_screenshot::prelude::*;

use glium::texture::RawImage2d;
use glium::{uniform, DrawParameters, Rect, Surface};
use imgui_glium_renderer::Renderer;
use savefile::{save_file, SavefileError};
use crate::generic_node_info::GenericNodeInfo;
use anyhow::anyhow;


use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};

use crate::nodes::node_enum::NodeType;

#[derive(Savefile)]
pub struct CaptureWindowNode {
    x: f32,
    y: f32,
    id: String,
    pub app_name: String,
    entire_screen: bool,
    pub hwnd: isize,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    data: Vec<u8>,
}

impl Default for CaptureWindowNode {
    fn default() -> Self {
        CaptureWindowNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            data: vec![],
            app_name: "".to_owned(),
            entire_screen: false,
            hwnd: 0,
        }
    }
}
impl MyNode for CaptureWindowNode {
    fn path(&self) -> Vec<&str> {
        vec!["Window"]
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn generic_info(&self) -> GenericNodeInfo {
        GenericNodeInfo {
            x: self.x,
            y: self.y,
            type_: self.type_(),
            id: self.id.to_owned(),
        }
    }

    fn savefile_version() -> u32 {
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
     

    fn type_(&self) -> NodeType {
        NodeType::CaptureDesktop
    }

     

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            CaptureWindowNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec![];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Out".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, _renderer: &mut Renderer, _storage: &Storage) {
        ui.checkbox("capture entire screen", &mut self.entire_screen);

        ui.disabled(self.entire_screen, || {
            ui.input_text("program name", &mut self.app_name).build();

            if ui.is_item_edited() {
                let re = Regex::new(&self.app_name).unwrap_or(Regex::new(r"~~~~error~~~").unwrap());
                let hwnd = window_list()
                    .unwrap()
                    .iter()
                    .find(|i| re.is_match(&i.window_name))
                    .unwrap_or(&HwndName {
                        hwnd: 0,
                        window_name: "error".to_owned(),
                    })
                    .hwnd;
                self.hwnd = hwnd;
            }

            let mut hwnd = self.hwnd as i32;
            ui.input_int("hwnd (window handle)", &mut hwnd).build();

            if ui.is_item_edited() {
                self.app_name = "".to_string()
            }

            self.hwnd = hwnd as isize;
        });
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        _map: HashMap<String, String>,
        _renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let output_id =self.output_id(&self.outputs()[0]);;

        let fragment_shader_src = include_str!("fix_desktop_capture.glsl");
        storage
            .gen_frag_shader(fragment_shader_src.to_string())
            .ok_or(anyhow!("failed to compile shader"))?;

        let buf = if self.entire_screen {
            match capture_display() {
                Ok(a) => a,
                Err(_) => {
                    return Err(anyhow!("failed to capture display"));
                }
            }
        } else {
            // match capture_window_ex(self.hwnd, Using::BitBlt, Area::ClientOnly, None, None) {
            //     Ok(a) => a,
            //     Err(_) => {return false;},
            // }
            match capture_window(self.hwnd) {
                Ok(a) => a,
                Err(_) => {
                    return Err(anyhow!("failed to capture window", ));
                }
            }
        };

        let size = (buf.width, buf.height);

        self.data = buf.pixels;

        storage.create_and_set_texture(size.0, size.1, output_id.clone());
        storage.create_and_set_texture(size.0, size.1, output_id.clone() + "-temp-texture");

        // let mut data: Vec<u8> = Vec::new();
        let texture2 = storage.get_texture(&(output_id)).unwrap();
        let texture = storage.get_texture(&(output_id + "-temp-texture")).unwrap();
        if self.data.len() as u32 == size.0 * size.1 * 4 {
            let image2d: RawImage2d<u8> = RawImage2d::from_raw_rgba(self.data.clone(), size);
            texture.write(
                Rect {
                    left: 0,
                    bottom: 0,
                    width: texture.width(),
                    height: texture.height(),
                },
                image2d,
            );
        } else {
            log::info!("incorrect size {:?} {}", size, self.data.len())
        }

        let uniforms = uniform! {
            tex: texture,
        };
        // glutin
        let shader = storage
            .get_frag_shader(fragment_shader_src.to_string())
            .unwrap();        texture2
            .as_surface()
            .draw(
                &storage.vertex_buffer,
                &storage.indices,
                shader,
                &uniforms,
                &DrawParameters {
                    ..Default::default()
                },
            )
            .unwrap();
        // std::thread::sleep(Duration::from_millis(5));

        return Ok(());
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("basic node, for debugging purposes")
    }
}
