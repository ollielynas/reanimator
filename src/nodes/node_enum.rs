use std::path::PathBuf;

use basic_shader_nodes::invert::InvertTextureNode;
use basic_shader_nodes::shader_generic::GenericShaderNode;
use basic_shader_nodes::solid_color::ColorNode;
use combine_rgba::CombineRgbaNode;
use debug::DebugNode;
use default_image::DefaultImage;
use frame_delay::DelayNode;
use image_io::OutputNode;
use load_gif::LoadGifNode;
use load_image::LoadImage;
use pick_random::RandomInputNode;
use restrict_pallet::RestrictPalletNode;
use savefile::{self, SavefileError};
use split_rgba::SplitRgbaNode;
use strum_macros::EnumIter;

use crate::node::MyNode;

use crate::nodes::*;



#[derive(Savefile, EnumIter, PartialEq, Eq, Copy, Clone, Debug)]
pub enum NodeType {
    Debug,
    Output,
    DefaultImageOut,
    InvertTexture,
    VHS,
    ChromaticAberration,
    LoadImageType,
    RestrictPalletRGBA,
    RandomInput,
    LoadGif,
    SplitRgba,
    Delay,
    CombineRgba,
    SolidColor,
}

impl NodeType  {
    pub fn name(&self) -> String {
        match self {
            NodeType::Debug => "Debug",
            NodeType::Output => "Output",
            NodeType::DefaultImageOut => "Default Image",
            NodeType::InvertTexture => "Invert Texture",
            NodeType::ChromaticAberration => "Chromatic Aberration",
            NodeType::VHS => "VHS",
            NodeType::LoadImageType => "Load Image",
            NodeType::RestrictPalletRGBA => "Restrict RGBA Pallet",
            NodeType::RandomInput => "Pick Random",
            NodeType::LoadGif => "Load Gif",
            NodeType::Delay => "Delay",
            NodeType::SplitRgba => "Split RGBA Channels",
            NodeType::CombineRgba => "Combine RGBA Channels",
            NodeType::SolidColor => "Solid Color",
        }.to_owned()
    }

    pub fn load_node(&self, project_file: PathBuf) -> Option<Box<dyn MyNode>>  {
        
        match self {
            NodeType::SolidColor => {
                let a: Result<ColorNode, SavefileError> = savefile::load_file(project_file, CombineRgbaNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            }
            NodeType::CombineRgba => {
                let a: Result<CombineRgbaNode, SavefileError> = savefile::load_file(project_file, CombineRgbaNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::SplitRgba => {
                let a: Result<SplitRgbaNode, SavefileError> = savefile::load_file(project_file, SplitRgbaNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::Delay => {
                let a: Result<DelayNode, SavefileError> = savefile::load_file(project_file, DelayNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::Debug => {
                let a: Result<DebugNode, SavefileError> = savefile::load_file(project_file, DebugNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::RandomInput => {
                let a: Result<RandomInputNode, SavefileError> = savefile::load_file(project_file, RandomInputNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::RestrictPalletRGBA => {
                let a: Result<RestrictPalletNode, SavefileError> = savefile::load_file(project_file, RestrictPalletNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::Output => {
                let a: Result<OutputNode, SavefileError> = savefile::load_file(project_file, OutputNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::LoadImageType => {
                let a: Result<LoadImage, SavefileError> = savefile::load_file(project_file, LoadImage::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::LoadGif => {
                let a: Result<LoadGifNode, SavefileError> = savefile::load_file(project_file, LoadGifNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::DefaultImageOut => {
                let a: Result<DefaultImage, SavefileError> = savefile::load_file(project_file, DefaultImage::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::InvertTexture => {
                let a: Result<InvertTextureNode, SavefileError> = savefile::load_file(project_file, InvertTextureNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            },
            NodeType::ChromaticAberration | NodeType::VHS => {
                let a: Result<GenericShaderNode, SavefileError> = savefile::load_file(project_file, GenericShaderNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            }
        }
    }

    pub fn new_node(self) -> Box<dyn MyNode>  where Self: Sized {
        match self {
            NodeType::SolidColor => Box::new(ColorNode::default()),
            NodeType::Debug => Box::new(DebugNode::default()),
            NodeType::Output => Box::new(OutputNode::default()),
            NodeType::DefaultImageOut => Box::new(DefaultImage::default()),
            NodeType::InvertTexture => Box::new(InvertTextureNode::default()),
            NodeType::VHS => Box::new(GenericShaderNode::new(NodeType::VHS)),
            NodeType::ChromaticAberration => Box::new(GenericShaderNode::new(NodeType::ChromaticAberration)),
            NodeType::LoadImageType => Box::new(LoadImage::default()),
            NodeType::RestrictPalletRGBA => Box::new(RestrictPalletNode::new()),
            NodeType::RandomInput => Box::new(RandomInputNode::default()),
            NodeType::LoadGif => Box::new(LoadGifNode::default()),
            NodeType::SplitRgba => Box::new(SplitRgbaNode::default()),
            NodeType::Delay => Box::new(DelayNode::default()),
            NodeType::CombineRgba => Box::new(CombineRgbaNode::default()),
            }
    }
}