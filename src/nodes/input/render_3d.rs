use crate::{
    node::{random_id, MyNode},
    storage::Storage,
};
use glium::{texture::RawImage2d, Texture2d};
use image::EncodableLayout;
use image::{self, ImageFormat};
use imgui::text_filter;
use imgui_glium_renderer::Renderer;
use lumo::tracer::*;
use lumo::*;
use platform_dirs::AppDirs;
use rfd::FileDialog;
use savefile::{load_file, save_file, SavefileError};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::{any::Any, collections::HashMap, env::current_exe, fs::{self, remove_dir_all}, hash::Hash, path::PathBuf};

use crate::nodes::node_enum::NodeType;

use super::apply_path_root;

#[derive(Savefile)]
pub struct Render3DNode {
    x: f32,
    y: f32,
    id: String,
    pub mtl_path: Option<PathBuf>,
    pub obj_path: Option<PathBuf>,
    pub color: [f32; 3],

    height: u32,
    width: u32,

    
    material_type: MaterialType,

    v_scatter_param: f32,
    v_sigma_t: [f32;3],

    refraction_index: f32,


    room: bool,

    samples: i32,

    use_gaussen: bool,
    gaussen_alpha: f32,

    rotate: [f32;3],


    #[savefile_ignore]
    #[savefile_introspect_ignore]
    texture_cache: Option<u64>,

    #[savefile_versions="1.."]
    // #[savefile_default_val=""]
    render_data: Vec<u8>,
}
#[derive(Savefile, EnumIter, PartialEq, Eq, Clone, Copy)]
enum MaterialType {
    Mirror,
    Matt,
    Texture,
    Glass,
    Volume,
}


impl Default for MaterialType {
    fn default() -> Self {
        MaterialType::Matt
    }
}

impl Default for Render3DNode {
    fn default() -> Self {
        Render3DNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
            texture_cache: None,
            mtl_path: None,
            obj_path: None,
            render_data: vec![],
            room: true,
            height: 256,
            width: 256,
            color: [0.9; 3],
            rotate: [0.0; 3],

            samples: 3,

            v_scatter_param: 1.0,
            v_sigma_t: [1.0;3],

            refraction_index: 2.0,

            use_gaussen: true,
            gaussen_alpha: 1.0,
            

            material_type: MaterialType::default(),
        }
    }
}

impl Render3DNode {
    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.obj_path.is_none() {return Ok(())}

        let camera = Camera::default(self.width as i32, self.height as i32);

        let def_color = Color::new(242, 242, 242);
        
        // let mut scene = Scene::default();
        let mut scene = Scene::default();

        if self.room {
            scene = Scene::empty_box(
                def_color,
                Material::diffuse(Texture::Solid(Color::new(255, 0, 0))),
                Material::diffuse(Texture::Solid(Color::new(0, 255, 0))),
            );
        }else {
            scene.add_light(Sphere::new(
                8.0 * Vec3::Y + 2.5 * Vec3::NEG_Z,
                4.0,
                Material::Light(Texture::Solid(Color::WHITE)),
            ));
        }

        

        let mut im_buffer = vec![];

        for i in self.render_data.chunks(3) {
            if i.len() < 3 {
                break;
            }
            im_buffer.push(
                Color::new(i[0], i[1], i[2])
            )
        }

        

        let color = Color {
            rgb: Vec3 { x: self.color[0] as f64, y:  self.color[1] as f64, z:  self.color[2] as f64 }
        };
        let sigma_color = 
            Vec3 { x: self.color[0] as f64, y:  self.color[1] as f64, z:  self.color[2] as f64 };
        

        let mat = match self.material_type {
            MaterialType::Mirror => Material::Mirror,
            MaterialType::Matt => Material::Lambertian(Texture::Solid(color)),
            MaterialType::Texture => Material::Lambertian(Texture::Image(Image { buffer: im_buffer, width: self.width, height: self.height })),
            MaterialType::Glass => Material::Glass(self.refraction_index as f64),
            MaterialType::Volume => Material::Volumetric(self.v_scatter_param as f64, sigma_color, color)
        };

        let mut obj = parser::mesh_from_path(
            <Option<PathBuf> as Clone>::clone(&self.obj_path).unwrap().to_str().unwrap_or(""),
            mat,
        )
        .unwrap();

        // let bounds = obj.bounding_box();



        scene.add(
            obj.to_unit_size()
            .to_origin()
            .rotate_x(self.rotate[0] as f64)
            .rotate_y(self.rotate[1] as f64)
            .rotate_x(self.rotate[2] as f64)
            .translate(0.0, 0.0, -1.3)
        );



        // scene.add_light();

        

        let mut renderer: lumo::Renderer = lumo::Renderer::new(scene, camera);

        if self.use_gaussen {
        renderer.set_filter(Filter::Gaussian(self.gaussen_alpha as f64));
        }else {
            renderer.set_filter(Filter::Box)
        }
        renderer.set_samples(self.samples as i32);

        let film: Film = renderer.render();


        let temp = match AppDirs::new(Some("Reanimator"), false) {
            Some(a) => {
                fs::create_dir_all(a.config_dir.clone());
                a.config_dir
            }
            None => current_exe().unwrap(),
        }.join("temp");

        fs::create_dir_all(&temp);

        let img_path = temp.join(self.id()+".png");

        film.save(img_path.to_str().unwrap());

        let bytes = match fs::read(img_path) {
            Ok(a) => a,
            Err(e) => {
                println!("{e}");
                return Ok(());
            }
        };
        let image = match image::load_from_memory(&bytes) {
            Ok(a) => a,
            Err(e) => {
                println!("{e}");
                return Ok(());
            }
        }
        .flipv()
        .into_rgba8();

        self.render_data = image.into_vec();

        println!("data len: {}", self.render_data.len());

        remove_dir_all(temp);

        self.texture_cache = None;

        Ok(())
    }
}

impl MyNode for Render3DNode {
    fn savefile_version() -> u32
    where
        Self: Sized,
    {
        1
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn path(&self) -> Vec<&str> {
        vec!["IO", "Load"]
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn description(&mut self, ui: &imgui::Ui) {
        ui.text_wrapped("this node allows you to render .obj files");
  
    }

    fn edit_menu_render(&mut self, ui: &imgui::Ui, renderer: &mut Renderer, storage: &Storage) {

        ui.columns(3, "render col", true);

        let mut input_val = [self.width as i32,self.height as i32];
        ui.input_int2("dimensions (w,h)", &mut input_val).build();
        self.width = input_val[0].max(1) as u32;
        self.height = input_val[1].max(1) as u32;

        
        ui.text(format!(
            "object path: {}",
            match &self.obj_path {
                Some(a) => {
                    a.as_path().to_str().unwrap()
                }
                None => "no path selected",
            }
        ));

        if ui.button("change obj path") {
            self.obj_path = FileDialog::new().add_filter("", &["obj"]).pick_file();
            if let Some(ref mut path) = self.obj_path {
                apply_path_root::set_root(path, &storage);
            }
        }

        ui.next_column();

        let mut index = MaterialType::iter().enumerate().find_map(|(i,v)| if v == self.material_type {Some(i)} else {None}).unwrap_or(0);
        
        let items = &MaterialType::iter().collect::<Vec<MaterialType>>();

        ui.combo("Material", &mut index, &items, |x| {x.name().into()});
        ui.checkbox("room", &mut self.room);
        ui.input_float3("rotation (x,y,z)", &mut self.rotate).build();
        self.material_type = items[index];

        match self.material_type {
            MaterialType::Mirror => {},
            MaterialType::Matt => {
                ui.color_edit3("color", &mut self.color);
            },
            MaterialType::Texture => {
                ui.text(format!(
                    "material path: {}",
                    match &self.mtl_path {
                        Some(a) => {
                            a.as_path().to_str().unwrap()
                        }
                        None => "no path selected",
                    }
                ));
        
                if ui.button("change texture path") {
                    self.mtl_path = FileDialog::new().add_filter("", &["png"]).pick_file();
                }


            },
            MaterialType::Glass => {
                ui.input_float("refraction index", &mut self.refraction_index).build();
            },
            MaterialType::Volume => {
                ui.input_float("refraction index", &mut self.refraction_index).build();
                ui.input_float3("sigma t", &mut self.v_sigma_t).build();
                ui.color_edit3("color", &mut self.color);
            },
        }

        ui.next_column();

        ui.checkbox("Use Gaussen blur", &mut self.use_gaussen);
        ui.disabled(!self.use_gaussen, || {
            ui.input_float("Gaussen Alpha", &mut self.gaussen_alpha).build();
            
        });

        ui.input_int("samples per pixel", &mut self.samples).build();
        self.samples= self.samples.max(1);

        if ui.button("render") {
            let a = self.render();

            if a.is_err() {
                println!("{a:?}");
            }
        }

    }

    fn type_(&self) -> NodeType {
        NodeType::Render3D
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
        return save_file(
            path.join(self.name()).join(self.id() + ".bin"),
            Render3DNode::savefile_version(),
            self,
        );
    }

    fn inputs(&self) -> Vec<String> {
        vec![]
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Selected Image".to_string()];
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
        

        let output_id = self.output_id(self.outputs()[0].clone());

        // println!("{:?}", self.texture_cache);

        if self.render_data.len() > 0 {
            if self.texture_cache.is_none()
                || !storage.cached_texture_exists(self.texture_cache.unwrap())
            {
                let not_texture =
                    RawImage2d::from_raw_rgba(self.render_data.clone(), (self.width, self.height));
                // let a: HashMap<Texture2d, String> = HashMap::new();
                let texture: Texture2d = Texture2d::new(&storage.display, not_texture).unwrap();
                self.texture_cache = Some(storage.cache_texture(texture));
            } else {
                // return false;
            }
        }
        if self.texture_cache.is_some() {
        storage.set_id_of_cached_texture(self.texture_cache.unwrap(), output_id);
    }
        // storage.set_texture(output_id,  texture);

        return true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}


impl MaterialType {
    fn name(&self) -> String {
        match self {
            MaterialType::Mirror => "Mirror",
            MaterialType::Matt => "Matt Color",
            MaterialType::Texture => "Texture",
            MaterialType::Glass => "Glass",
            MaterialType::Volume => "Volume",
        }.to_owned()
    }
}