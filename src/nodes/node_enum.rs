use std::path::PathBuf;

use basic_shader_nodes::invert::InvertTextureNode;
use default_image::DefaultImage;
use image_io::OutputNode;
use savefile::{self, SavefileError};
use strum_macros::EnumIter;

use crate::node::{DebugNode, MyNode, VERSION};

use crate::nodes::*;



#[derive(Savefile, EnumIter)]
pub enum NodeType {
    Debug,
    Output,
    DefaultImageOut,
    InvertTexture,
}

impl NodeType  {
    pub fn name(&self) -> String {
        match self {
            NodeType::Debug => "Debug",
            NodeType::Output => "Output",
            NodeType::DefaultImageOut => "Default Image",
            NodeType::InvertTexture => "Invert Texture",
        }.to_owned()
    }

    pub fn load_node(&self, node_id: String, project_file: PathBuf) -> Option<Box<dyn MyNode>>  {
        match self {
            NodeType::Debug => {
                let a: Result<DebugNode, SavefileError> = savefile::load_file(project_file, VERSION);
                match a {Ok(b) => Some(Box::new(b)), Err(a) => None}
            },
            NodeType::Output => {
                let a: Result<OutputNode, SavefileError> = savefile::load_file(project_file, VERSION);
                match a {Ok(b) => Some(Box::new(b)), Err(a) => None}
            },
            NodeType::DefaultImageOut => {
                let a: Result<DefaultImage, SavefileError> = savefile::load_file(project_file, VERSION);
                match a {Ok(b) => Some(Box::new(b)), Err(a) => None}
            },
            NodeType::InvertTexture => {
                let a: Result<InvertTextureNode, SavefileError> = savefile::load_file(project_file, VERSION);
                match a {Ok(b) => Some(Box::new(b)), Err(a) => None}
            },
        }
    }

    pub fn new_node(self) -> Box<dyn MyNode>  where Self: Sized {
        match self {
            NodeType::Debug => Box::new(DebugNode::default()),
            NodeType::Output => Box::new(OutputNode::default()),
            NodeType::DefaultImageOut => Box::new(DefaultImage::default()),
            NodeType::InvertTexture => Box::new(InvertTextureNode::default())
            }
    }
}