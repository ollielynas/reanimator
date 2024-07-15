use std::{any::Any, collections::HashMap, iter::Filter, path::{self, PathBuf}, process::Output};
use std::hash::Hash;
use fastrand;
use glium::{uniform, BlitTarget, DrawParameters, Surface, Texture2d};
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use savefile::prelude::*;

use crate::{
    nodes::{image_io::OutputNode, node_enum::NodeType},
    storage::Storage,
};



pub trait MyNode {

    fn savefile_version() -> u32 where Self: Sized;

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn path(&self) -> Vec<&str>;

    fn type_(&self) -> NodeType;

    fn y(&self) -> f32;
    fn x(&self) -> f32;

    fn name(&self) -> String {
        self.type_().name()
    }
    fn id(&self) -> String;

    // fn set_pos();

    fn inputs(&self) -> Vec<String>;
    fn outputs(&self) -> Vec<String>;

    fn set_xy(&mut self, x: f32, y: f32);

    fn edit_menu_render(&mut self, ui: &Ui, renderer: &mut Renderer) {
        ui.text("this node cannot be edited");
    }
    fn description(&mut self, ui: &Ui) {
        ui.text("this node does not yet have a description");
    }

    /// # Use This
    /// ```
    /// fn save(&self, path: PathBuf) -> Result<(), SavefileError> {
    ///     return save_file(
    ///         path.join(self.name()).join(self.id()+".bin"),
    ///         NodeStruct::savefile_version(),
    ///         self,
    ///     );
    /// }
    /// ```
    fn save(&self, path: PathBuf) -> Result<(), SavefileError>;

    fn input_id(&self, input: String) -> String {
        format!("node-{}-input-{input}", self.id())
    }
    fn output_id(&self, output: String) -> String {
        format!("node-{}-output-{output}", self.id())
    }

    fn set_id(&mut self, id: String);

    fn run(&mut self, storage: &mut Storage, map: HashMap<String, String>, renderer: &mut Renderer) -> bool;
}



pub fn random_id() -> String {
    fastrand::i32(1000..=9999).to_string()
}
