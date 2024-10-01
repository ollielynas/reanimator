use std::{any::Any, collections::HashMap, os::raw::c_void, path::PathBuf, rc::Rc};

use glium::{
    uniforms::{MagnifySamplerFilter, SamplerBehavior},
    BlitTarget, Rect, Surface, Texture2d,
};
use imgui::{TextureId, Ui};
use imgui_glium_renderer::{Renderer, Texture};
use imgui_winit_support::winit::{
    dpi::{Position, Size},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
};
use savefile::{save_file, SavefileError};
use anyhow::anyhow;

use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::ClientToScreen,
    UI::WindowsAndMessaging::{GetClientRect, GetForegroundWindow},
};
// use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use crate::{
    node::{random_id, MyNode},
    nodes::node_enum::NodeType,
    storage::Storage,
};

use regex::Regex;
use win_screenshot::prelude::*;

#[derive(Savefile)]
pub struct CoverWindowNode {
    x: f32,
    y: f32,
    id: String,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub texture: Option<Texture2d>,
    pub app_name: String,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub hwnd: isize,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub render: bool,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub texture_id: Option<TextureId>,
}

impl Default for CoverWindowNode {
    fn default() -> Self {
        CoverWindowNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            texture: None,
            app_name: String::new(),
            hwnd: 0,
            render: false,
            texture_id: None,
        }
    }
}

impl CoverWindowNode {
    pub fn render(
        &mut self,
        ui: &Ui,
        window: &imgui_winit_support::winit::window::Window,
        storage: &mut Storage,
        renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        if self.hwnd == 0 {
            return Err(anyhow!("invalid window id"));
        }

        if self.render {
            let mut rect = RECT::default();
            unsafe {
                let a = GetClientRect(HWND(self.hwnd as *mut _), &mut rect);
                if a.is_err() {
                    self.render = false;
                    // log::info!("{a:?}");
                    return Err(anyhow!("invalid window id"));
                }
                let mut point = POINT {
                    x: rect.left,
                    y: rect.top,
                };
                let a = ClientToScreen(HWND(self.hwnd as *mut _), &mut point);
                if !a.as_bool() {
                    self.render = false;
                    return Err(anyhow!("failed 'ClientToScreen' conversion, likely because of an invalid window id"));
                }
                rect.left = point.x;
                rect.top = point.y;
            }

            let steam_focused = unsafe { GetForegroundWindow().0 == self.hwnd as *mut _ };

            if steam_focused {
                let _ = window.request_inner_size(Size::Physical(
                    (rect.right as f64, rect.bottom as f64).into(),
                ));
                let _ = window.set_cursor_hittest(false);
                window.set_decorations(false);
                window.set_resizable(false);
                window.set_transparent(true);

                window
                    .set_window_level(imgui_winit_support::winit::window::WindowLevel::AlwaysOnTop);
                window.set_outer_position(Position::Physical((rect.left, rect.top).into()));

                if self.texture_id.is_none() {
                    self.texture_id = Some(renderer.textures().insert(Texture {
                        texture: Rc::new(Texture2d::empty(&storage.display, 10, 10).unwrap()),
                        sampler: SamplerBehavior {
                            // minify_filter: MinifySamplerFilter:,
                            magnify_filter: MagnifySamplerFilter::Nearest,
                            ..Default::default()
                        },
                    }));
                }

                if let Some(frame) = &self.texture {
                    if let Some(texture_id) = self.texture_id {
                        if let Some(texture) = renderer.textures().get_mut(texture_id) {
                            if texture.texture.dimensions() != frame.dimensions() {
                                texture.texture = Rc::new(
                                    Texture2d::empty(
                                        &storage.display,
                                        frame.width(),
                                        frame.height(),
                                    )
                                    .unwrap(),
                                );
                            }

                            // let simple_frame_buffer = SimpleFrameBuffer::new(&storage.display, ColorA);
                            frame.as_surface().blit_color(
                                &Rect {
                                    left: 0,
                                    bottom: 0,
                                    width: frame.width(),
                                    height: frame.height(),
                                },
                                &texture.texture.as_surface(),
                                &BlitTarget {
                                    left: 0,
                                    bottom: texture.texture.height(),
                                    width: texture.texture.width() as i32,
                                    height: -(texture.texture.height() as i32),
                                },
                                MagnifySamplerFilter::Nearest,
                            );

                            ui.get_foreground_draw_list()
                                .add_image(texture_id, [0.0, 0.0], ui.io().display_size)
                                .build();
                        }
                        return Ok(());
                    } else {
                        return Ok(());
                    }
                } else {
                    ui.text("failed to load get texture");
                    return Ok(());
                }
            } else {
                return Err(anyhow!("target window is not in focus"));
            }
        } else {
            return Err(anyhow!("rendering is disabled"));
        }
    }
}

impl MyNode for CoverWindowNode {
    fn path(&self) -> Vec<&str> {
        vec!["Window"]
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
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
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::CoverWindow
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            CoverWindowNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec![];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn edit_menu_render(&mut self, ui: &Ui, _renderer: &mut Renderer, storage: &Storage) {
        ui.checkbox("enable", &mut self.render);

        if ui.is_item_hovered() {
            ui.tooltip_text("will take effect when window is focused");
        }

        ui.input_text("program name", &mut self.app_name).build();

        if ui.is_item_edited() {
            let re = Regex::new(&self.app_name).unwrap_or(Regex::new(r"~~~~error~~~").unwrap());
            let hwnd = window_list()
                .unwrap()
                .iter()
                .find(|i| re.is_match(&i.window_name) && !i.window_name.contains("ReAnimator"))
                .unwrap_or(&HwndName {
                    hwnd: 0,
                    window_name: "error".to_owned(),
                })
                .hwnd;
            if self.app_name.len() >= 2 {
                self.hwnd = hwnd;
            } else {
                self.hwnd = 0;
            }
        }

        let mut hwnd = self.hwnd as i32;
        ui.input_int("hwnd (window handle)", &mut hwnd).build();

        if ui.is_item_edited() {
            self.app_name = "".to_string()
        }

        self.hwnd = hwnd as isize;
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        renderer: &mut Renderer,
    ) -> anyhow::Result<()> {
        let input_id = self.input_id(&self.inputs()[0]);
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => return  Err(anyhow!("missing input")),
        };

        let texture: &glium::Texture2d = match storage.get_texture(get_output) {
            Some(a) => a,
            None => return Err(anyhow!("failed to get input texture from storage")),
        };

        if self.texture.is_none() {
            self.texture = Some(
                Texture2d::empty(&storage.display, texture.width(), texture.height()).unwrap(),
            );
        }

        if let Some(texture2) = &self.texture {
            texture.as_surface().blit_color(
                &Rect {
                    left: 0,
                    bottom: 0,
                    height: texture.height(),
                    width: texture.width(),
                },
                &texture2.as_surface(),
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    height: texture2.height() as i32,
                    width: texture2.width() as i32,
                },
                glium::uniforms::MagnifySamplerFilter::Linear,
            );
        } else {
            // self.texture = Texture2d::empty(, width, height)
        }

        return Ok(());
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("replaces the texture of the window that it has been set to cover with the imputed texture\n Press Ctrl Esc or Shift Esc to disable");
    }
}
