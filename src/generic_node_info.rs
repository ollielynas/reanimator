use std::str::FromStr;

use strum::IntoEnumIterator;

use crate::{nodes::node_enum::NodeType};


pub struct GenericNodeInfo {
    pub x: f32,
    pub y: f32,
    pub type_: NodeType,

    pub id: String,
}

impl FromStr for NodeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for i in NodeType::iter() {
            if &i.to_string() == s {
                return Ok(i);
            }
        }
        return Err(format!("{s} is not a valid enum"));
    }
}


