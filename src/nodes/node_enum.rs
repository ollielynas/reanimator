use std::path::PathBuf;

use basic_shader_nodes::*;
use basic_shader_nodes::invert::InvertTextureNode;
use basic_shader_nodes::shader_generic::GenericShaderNode;
use basic_shader_nodes::solid_color::ColorNode;
use detect_motion::MotionNode;
use difference_of_gaussians::DifferenceofGaussiansNode;
use input::default_image::DefaultImage;
use input::desktop_capture::CaptureWindowNode;
use input::load_gif::LoadGifNode;
use input::load_image::LoadImage;
use input::load_video::LoadVideoNode;
use input::render_3d::Render3DNode;
use input::webcam::WebcamNode;
use mask::layer_trail::LayerTrailNode;
use mask::text_mask::TextMaskNode;

use cover_window::CoverWindowNode;
use debug::DebugNode;
use dither::{BayerDitherNode, LinearErrorDitherNode};
use frame_delay::DelayNode;
use glium::Display;
use image_io::OutputNode;
use layer::LayerNode;
use mask::color_noise::ColorNoiseNode;
use mask::generic_mask::GenericMaskNode;
use mask::multiply::MultiplyNode;
use mask::white_noise::WhiteNoiseNode;
use pick_random::RandomInputNode;
use regex::Regex;
use restrict_pallet::RestrictPalletNode;
use rgb_hsl::combine_hsv::*;
use rgb_hsl::combine_rgba::CombineRgbaNode;
use rgb_hsl::split_hsv::SplitHsvNode;
use rgb_hsl::split_rgba::SplitRgbaNode;
use savefile::{self, SavefileError};
use scale::ScaleNode;

use strum_macros::EnumIter;
use text::display_text::DisplayTextNode;
use text::text_input::TextInputNode;
use watercolor::watercolor::WaterColorNode;
use win_screenshot::utils::{window_list, HwndName};

use crate::node::MyNode;

use crate::nodes::*;



#[derive(Savefile, EnumIter, PartialEq, Eq, Copy, Clone, Debug, Hash)]
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
    Multiply,
    WhiteNoise,
    Layer,
    ColorNoise,
    Blur,
    Render3D,
    BrightnessMask,
    DifferenceOfGaussians,
    Webcam,
    Dot,
    TextMask,
    TextInput,
    SplitHsv,
    CombineHsv,
    Scale,
    LinearErrorDither,
    BayerDither,
    Sharpness,
    CaptureDesktop,
    CoverWindow,
    DisplayText,
    Motion,
    BlurSp,
    LayerTrail,
    WaterColor,
    LoadVideo,
}

impl NodeType  {
    pub fn name(&self) -> String {
        match self {
            NodeType::Render3D => "3D Render (Ray Traced)",
            NodeType::Layer => "Layer",
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
            NodeType::SplitRgba => "Split RGBA",
            NodeType::CombineRgba => "Combine RGBA",
            NodeType::SolidColor => "Solid Color",
            NodeType::Multiply => "Multiply",
            NodeType::WhiteNoise => "White Noise",
            NodeType::ColorNoise => "Color Noise",
            NodeType::Blur => "Blur",
            NodeType::BrightnessMask => "Mask Brightness",
            NodeType::DifferenceOfGaussians => "Difference of Gaussians",
            NodeType::Webcam => "Webcam Input",
            NodeType::Dot => "Dots",
            NodeType::TextMask => "Text Mask",
            NodeType::TextInput => "Text Input",
            NodeType::SplitHsv => "Split HSV",
            NodeType::CombineHsv => "Combine HSV",
            NodeType::Scale => "Resize",
            NodeType::LinearErrorDither => "Error Diffusion Dither",
            NodeType::BayerDither => "Bayer Dither",
            NodeType::Sharpness => "Sharpness",
            NodeType::CaptureDesktop => "Capture Window",
            NodeType::CoverWindow => "Cover Window",
            NodeType::DisplayText => "Display Text",
            NodeType::Motion => "Detect Motion",
            NodeType::BlurSp => "blursp", 
            NodeType::LayerTrail => "Layer Trail", 
            NodeType::WaterColor => "WaterColor", 
            NodeType::LoadVideo => "Load With ffmpeg", 
        }.to_owned()
    }

    pub fn load_node(&self, project_file: PathBuf) -> Option<Box<dyn MyNode>>  {
        
        match self {
            NodeType::TextInput => {
                let a: Result<TextInputNode, SavefileError> = savefile::load_file(project_file, TextInputNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::LoadVideo => {
                let a: Result<LoadVideoNode, SavefileError> = savefile::load_file(project_file, LoadVideoNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::WaterColor => {
                let a: Result<WhiteNoiseNode, SavefileError> = savefile::load_file(project_file, WhiteNoiseNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::LayerTrail => {
                let a: Result<LayerTrailNode, SavefileError> = savefile::load_file(project_file, LayerTrailNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::Motion => {
                let a: Result<MotionNode, SavefileError> = savefile::load_file(project_file, MotionNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::DisplayText => {
                let a: Result<DisplayTextNode, SavefileError> = savefile::load_file(project_file, DisplayTextNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::CaptureDesktop => {
                let a: Result<CaptureWindowNode, SavefileError> = savefile::load_file(project_file, CaptureWindowNode::savefile_version());
                
                match a {Ok(mut b) => {
                    let re = Regex::new(&b.app_name).unwrap_or(Regex::new(r"~~~~error~~~").unwrap());
    let hwnd = window_list()
        .unwrap()
        .iter()
        .find(|i| re.is_match(&i.window_name))
        .unwrap_or(&HwndName {hwnd: 0, window_name: "error".to_owned()})
        .hwnd;
            b.hwnd = hwnd;
                    Some(Box::new(b))
                }, Err(e) => {
                    println!("{e}");
                    None}}
                
            }
            NodeType::CoverWindow => {
                let a: Result<CoverWindowNode, SavefileError> = savefile::load_file(project_file, CoverWindowNode::savefile_version());
                match a {Ok(mut b) => {
                
                
                    let re = Regex::new(&b.app_name).unwrap_or(Regex::new(r"~~~~error~~~").unwrap());
    let hwnd = window_list()
        .unwrap()
        .iter()
        .find(|i| re.is_match(&i.window_name))
        .unwrap_or(&HwndName {hwnd: 0, window_name: "error".to_owned()})
        .hwnd;
            b.hwnd = hwnd;
                    Some(Box::new(b))
                }, Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::BayerDither => {
                let a: Result<BayerDitherNode, SavefileError> = savefile::load_file(project_file, BayerDitherNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::LinearErrorDither => {
                let a: Result<LinearErrorDitherNode, SavefileError> = savefile::load_file(project_file, LinearErrorDitherNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::Scale => {
                let a: Result<ScaleNode, SavefileError> = savefile::load_file(project_file, ScaleNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::CombineHsv => {
                let a: Result<CombineHsvNode, SavefileError> = savefile::load_file(project_file,  CombineHsvNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::SplitHsv => {
                let a: Result<SplitHsvNode, SavefileError> = savefile::load_file(project_file, SplitHsvNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::TextMask => {
                let a: Result<TextMaskNode, SavefileError> = savefile::load_file(project_file, TextMaskNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::Render3D => {
                let a: Result<Render3DNode, SavefileError> = savefile::load_file(project_file, Render3DNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::Webcam => {
                let a: Result<WebcamNode, SavefileError> = savefile::load_file(project_file, WebcamNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::DifferenceOfGaussians => {
                let a: Result<DifferenceofGaussiansNode, SavefileError> = savefile::load_file(project_file, DifferenceofGaussiansNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::Layer => {
                let a: Result<LayerNode, SavefileError> = savefile::load_file(project_file, LayerNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::ColorNoise => {
                let a: Result<ColorNoiseNode, SavefileError> = savefile::load_file(project_file, ColorNoiseNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }


            NodeType::WhiteNoise => {
                let a: Result<WhiteNoiseNode, SavefileError> = savefile::load_file(project_file, WhiteNoiseNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(e) => {
                    println!("{e}");
                    None}}
            }
            NodeType::Multiply => {
                let a: Result<MultiplyNode, SavefileError> = savefile::load_file(project_file, MultiplyNode::savefile_version());
                match a {Ok(b) => Some(Box::new(b)), Err(_) => None}
            }
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
            NodeType::ChromaticAberration 
            | NodeType::VHS 
            | NodeType::Blur
            | NodeType::Dot
            | NodeType::Sharpness
            | NodeType::BlurSp
             => {
                let a: Result<GenericShaderNode, SavefileError> = savefile::load_file(project_file, GenericShaderNode::savefile_version());
                match a {Ok(mut b) => {
                    b.load_type();
                    Some(Box::new(b))
                }, Err(_) => None}

                
            }
            NodeType::BrightnessMask => {
                let a: Result<GenericMaskNode, SavefileError> = savefile::load_file(project_file, GenericMaskNode::savefile_version());
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
            NodeType::VHS
            | NodeType::ChromaticAberration
            | NodeType::Blur
            | NodeType::Dot
            | NodeType::Sharpness
            | NodeType::BlurSp
             => 
            {Box::new(GenericShaderNode::new(self))},
            NodeType::BrightnessMask => 
            {
                Box::new(GenericMaskNode::new(self))
            },
            NodeType::LoadImageType => Box::new(LoadImage::default()),
            NodeType::RestrictPalletRGBA => Box::new(RestrictPalletNode::new()),
            NodeType::RandomInput => Box::new(RandomInputNode::default()),
            NodeType::LoadGif => Box::new(LoadGifNode::default()),
            NodeType::SplitRgba => Box::new(SplitRgbaNode::default()),
            NodeType::Delay => Box::new(DelayNode::default()),
            NodeType::CombineRgba => Box::new(CombineRgbaNode::default()),
            NodeType::Multiply => Box::new(MultiplyNode::default()),
            NodeType::WhiteNoise => Box::new(WhiteNoiseNode::default()),
            NodeType::Layer => Box::new(LayerNode::default()),
            NodeType::ColorNoise => Box::new(ColorNoiseNode::default()),
            NodeType::Render3D => Box::new(Render3DNode::default()),
            NodeType::DifferenceOfGaussians => Box::new(DifferenceofGaussiansNode::default()),
            NodeType::Webcam => Box::new(WebcamNode::default()),
            NodeType::TextMask => Box::new(TextMaskNode::default()),
            NodeType::TextInput => Box::new(TextInputNode::default()),
            NodeType::SplitHsv => Box::new(SplitHsvNode::default()),
            NodeType::CombineHsv => Box::new(CombineHsvNode::default()),
            NodeType::Scale => Box::new(ScaleNode::default()),
            NodeType::LinearErrorDither => Box::new(LinearErrorDitherNode::default()),
            NodeType::BayerDither => Box::new(BayerDitherNode::default()),
            NodeType::CaptureDesktop => Box::new(CaptureWindowNode::default()),
            NodeType::CoverWindow => Box::new(CoverWindowNode::default()),
            NodeType::DisplayText => Box::new(DisplayTextNode::default()),
            NodeType::Motion => Box::new(MotionNode::default()),
            NodeType::LayerTrail => Box::new(LayerTrailNode::default()),
            NodeType::WaterColor => Box::new(WaterColorNode::default()),
            NodeType::LoadVideo => Box::new(LoadVideoNode::default()),
            }
    }


    pub fn disabled(&self) -> bool {
        matches!(
            self,
            | NodeType::Debug
            | NodeType::Webcam  
            | NodeType::BlurSp
            | NodeType::TextInput
            | NodeType::TextMask
            | NodeType::DisplayText
            | NodeType::WaterColor
        )
    }

}