use fastrand;


use std::{
    any::Any,
    collections::HashMap,
    path::{PathBuf},
};
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use savefile::prelude::*;


use crate::{generic_node_info::GenericNodeInfo, nodes::node_enum::NodeType, render_nodes::RenderNodesParams, storage::Storage};

pub trait MyNode {

    fn savefile_version() -> u32
    where
        Self: Sized;

    fn generic_info(&self) -> GenericNodeInfo;


    /// if a node has some large one time cost then that cost should take place in this function
    fn load_assets(&mut self, _storage: &Storage) {}

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn path(&self) -> Vec<&str>;

    fn type_(&self) -> NodeType {
        self.generic_info().type_
    }

    fn y(&self) -> f32 {
        self.generic_info().y
    }
    fn x(&self) -> f32 {
        self.generic_info().x
    }

    fn name(&self) -> String {
        self.type_().name()
    }
    fn id(&self) -> String {
        self.generic_info().id
    }

    // fn set_pos();

    
    fn inputs(&self) -> Vec<String>;
    fn outputs(&self) -> Vec<String>;

    fn set_xy(&mut self, x: f32, y: f32);


    fn edit_menu_render(&mut self, ui: &Ui, _renderer: &mut Renderer, _storage: &Storage) {
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
    fn save(&self, path: PathBuf) -> Result<(), SavefileError> ;

    fn input_id(&self, input: &str) -> String {
        format!("node-{}-input-{input}", self.id())
    }

    fn output_id(&self, output: &str) -> String {
        format!("node-{}-output-{output}", self.id())
    }

    fn set_id(&mut self, id: String);

    fn render_in_node(&self, _ui: &Ui,ui_scale: f32,  _renderer: &mut Renderer, _params: &mut RenderNodesParams) {
        
    }

    fn run(
        &mut self,
        storage: &mut Storage,
        map: HashMap<String, String>,
        renderer: &mut Renderer,
    ) -> anyhow::Result<()>;
}

pub fn random_id() -> String {
    fastrand::i32(1000..=9999).to_string()
}
