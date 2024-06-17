use image_io::OutputNode;
use savefile::{self, SavefileError};
use strum_macros::EnumIter;

use crate::node::{DebugNode, MyNode, VERSION};

use crate::nodes::*;



#[derive(Savefile, EnumIter)]
pub enum NodeType {
    Debug,
    Output,
}

impl NodeType  {
    pub fn name(&self) -> String {
        match self {
            NodeType::Debug => "Debug",
            NodeType::Output => "Output"
        }.to_owned()
    }

    pub fn load_node(&self, node_id: String, project_file: String) -> Option<Box<dyn MyNode>>  {
        match self {
            NodeType::Debug => {
                let a: Result<DebugNode, SavefileError> = savefile::load_file(format!("{project_file}/nodes/{}/{}.bat", self.name(), node_id), VERSION);
                match a {Ok(b) => Some(Box::new(b)), Err(a) => None}
            },
            NodeType::Output => {
                let a: Result<DebugNode, SavefileError> = savefile::load_file(format!("{project_file}/nodes/{}/{}.bat", self.name(), node_id), VERSION);
                match a {Ok(b) => Some(Box::new(b)), Err(a) => None}
            },
        }
    }

    pub fn new_node(self) -> Box<dyn MyNode>  where Self: Sized {
        match self {
            NodeType::Debug => Box::new(DebugNode::default()),
            NodeType::Output => Box::new(OutputNode::default()),
            }
    }
}