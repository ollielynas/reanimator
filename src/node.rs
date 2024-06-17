use std::{collections::HashMap, process::Output};

use fastrand;
use savefile::prelude::*;

use crate::{
    nodes::{image_io::OutputNode, node_enum::NodeType},
    project::Storage,
};

pub const VERSION: u32 = 0;

pub fn random_id() -> String {
    fastrand::i32(1000..=9999).to_string()
}

pub trait MyNode {
    fn path(&self) -> Vec<&str>;

    fn type_(&self) -> NodeType;

    fn y(&self) -> f32;
    fn x(&self) -> f32;

    fn name(&self) -> String;
    fn id(&self) -> String;

    // fn set_pos();

    fn inputs(&self) -> Vec<String>;
    fn outputs(&self) -> Vec<String>;

    fn set_xy(&mut self, x: f32, y: f32);

    fn edit_menu_render(&self);

    fn save(&self) -> Result<(), SavefileError>;

    fn input_id(&self, input: String) -> String {
        format!("node-{}-input-{input}", self.id())
    }
    fn output_id(&self, input: String) -> String {
        format!("node-{}-input-{input}", self.id())
    }

    fn run(&self, storage: &mut Storage, map: HashMap<String, String>) -> bool;
}

#[derive(Savefile)]
pub struct DebugNode {
    x: f32,
    y: f32,
    id: String,
}

impl Default for DebugNode {
    fn default() -> Self {
        DebugNode {
            x: 0.0,
            y: 0.0,
            id: random_id(),
        }
    }
}

impl MyNode for DebugNode {
    fn path(&self) -> Vec<&str> {
        vec!["msc"]
    }

    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }

    fn type_(&self) -> NodeType {
        NodeType::Debug
    }

    fn name(&self) -> String {
        "debug".to_string()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn save(&self) -> Result<(), SavefileError> {
        return save_file(
            format!("./saves/{}/{}.bin", self.type_().name(), self.id()),
            VERSION,
            self,
        );
    }

    fn edit_menu_render(&self) {}

    fn inputs(&self) -> Vec<String> {
        return vec!["In".to_string()];
    }

    fn outputs(&self) -> Vec<String> {
        return vec!["Out".to_string()];
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn input_id(&self, input: String) -> String {
        format!("node-{}-input-{input}", self.id())
    }

    fn output_id(&self, input: String) -> String {
        format!("node-{}-input-{input}", self.id())
    }

    fn run(&self, storage: &mut Storage, map: HashMap::<String, String>) -> bool {
        let input_id = self.input_id(self.inputs()[0].clone());
        let output_id = self.output_id(self.outputs()[0].clone());
        let get_output = match map.get(&input_id) {
            Some(a) => a,
            None => {return false},
        };
        if let Some(frame) = storage.get_frame(get_output) {
            storage.set_frame(output_id, frame);
        }else {
            return false
        }
        return true;
    }
}
