use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};
use glium::{texture::RawImage2d, uniforms::MagnifySamplerFilter, BlitTarget, Rect, Surface, Texture2d};
use image::{self, ImageFormat};
use image::EncodableLayout;
use imgui::text_filter;
use imgui_glium_renderer::Renderer;
use rfd::FileDialog;
use savefile::{load_file, save_file, SavefileError};
use std::{any::Any, collections::HashMap, fs, hash::Hash, path::PathBuf};

use super::node_enum::NodeType;

use escapi;

#[derive(Savefile)]
pub struct WebcamNode {
    x: f32,
    y: f32,
    id: String,
    
    input: bool,
    size: (u32,u32),
    
    desired_fps: u64,

    #[savefile_ignore]
    #[savefile_introspect_ignore]
    #[savefile_default_val="999"]
    selected_cam: usize,


    main_webcam: String,

    #[savefile_ignore]
    #[savefile_introspect_ignore]
    available: Vec<String>,

    #[savefile_ignore]
    #[savefile_introspect_ignore]
    data: Vec<u8>,
    
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    camera: Option<escapi::Device>,
}

impl Default for WebcamNode {
    fn default() -> Self {
        let mut w = WebcamNode {
            x: 0.0,
            y: 0.0,
            data: vec![],
            camera: None,
            id: random_id(),
            main_webcam: String::new(),
            available: Vec::new(),
            size: (512,512),
            input: false,
            selected_cam: 999,
            desired_fps: 60,
        };
        w.load_webcams();
        return w;
    }
}

impl MyNode for WebcamNode {
    fn savefile_version() -> u32
    where
        Self: Sized,
    {
        0
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn path(&self) -> Vec<&str> {
        vec!["IO"]
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("Get webcam as input");
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer) {

        ui.checkbox("use input texture for dimensions", &mut self.input);
        ui.disabled(self.input, || {
            let mut input_val = [self.size.0 as i32,self.size.1 as i32];
            ui.input_int2("dimensions (w,h)", &mut input_val).build();
            self.size = (input_val[0].max(1) as u32, input_val[1].max(1) as u32);
        });

        ui.spacing();


        let before = self.selected_cam;
        ui.combo_simple_string("Cam Input", &mut self.selected_cam, &self.available);
        if before != self.selected_cam {
            self.load_webcams();
        }

        if ui.button("update input list") {
    // Query for available devices.
            self.load_webcams();
        }
    }

    fn type_(&self) -> NodeType {
        NodeType::Webcam
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            WebcamNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        if self.input {
            return vec!["In".to_string()];
        } else {
            return vec![];
        }
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Out".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        renderer: &mut Renderer,
    ) -> bool {
        return false;
        let output_id = self.output_id(self.outputs()[0].clone());

        // println!("{:?}", self.texture_cache);


            if let Some(cam) = &self.camera {
                self.size = (cam.capture_width(), cam.capture_height());
                self.data = match cam.capture() {
                    Ok(a) => a,
                    Err(_) =>{
                        return false
                },
                }.to_vec();
            };

            println!("size {:?}", self.size);

            storage.create_and_set_texture(self.size.0, self.size.1, output_id.clone());
            
            // println!("created texture");
            
            let texture: &glium::Texture2d = match storage.get_texture(&output_id) {
                Some(a) => a,
                None => return false,
            };
            
            // Texture2d::
            
            let raw_image: RawImage2d<u8> = RawImage2d::from_raw_rgba(self.data.clone(), self.size);
            // println!("created texture2");
            println!("{:?}",raw_image.data.len());

            if raw_image.data.len() == 0 {
                return false;
            }

            let texture2 = match Texture2d::new(&storage.display, raw_image) {
                Ok(a) => a,
                Err(e) => {
                    println!("{e}");
                    return false;
                },
            };
            
            
            texture.as_surface().blit_buffers_from_simple_framebuffer(&texture2.as_surface(), &Rect {
                left: 0,
                bottom: 0,
                width: texture2.width(),
                height: texture2.height(),
            }, 
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    width: texture.width() as i32,
                    height: texture.height() as i32,
                }
            , MagnifySamplerFilter::Linear, glium::BlitMask::color());

            // texture.
            



                // RawImage2d::from_raw_rgb(, dimensions);

                // let a: HashMap<Texture2d, String> = HashMap::new();
                // let texture: Texture2d = Texture2d::new(&storage.display, raw).unwrap();



        return true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}


impl WebcamNode {
    fn load_webcams(&mut self) {
        return;
        
        self.available = vec![];
        let num_of_devices = escapi::num_devices();
        for i in 0..num_of_devices {
            let cam = escapi::init(i, self.size.0, self.size.0, self.desired_fps);
            if let Ok(cam) = cam {
                println!("cam name: {}", cam.name());
            if self.selected_cam == 999 && cam.name() == self.main_webcam {
                self.selected_cam = i;
            }
            self.available.push(cam.name());
            if self.selected_cam == i {
                self.camera = Some(cam);
            }


            }else if let Err(e) = cam {
                self.available.push(format!("Error Loading {i}"));
                println!("{e}");
            }

        }
    }
}