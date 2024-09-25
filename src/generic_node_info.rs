use std::str::FromStr;

use strum::IntoEnumIterator;

use crate::{node::MyNode, nodes::node_enum::NodeType};

fn convert(type_: NodeType) -> String {
    type_.to_string()
}

#[derive(Savefile)]
pub struct GenericNodeInfo {
    pub x: f32,
    pub y: f32,
    #[savefile_versions_as = "0..0:convert:NodeType"]
    #[savefile_versions = "1.."]
    pub type_: String,

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

impl GenericNodeInfo {
    pub fn savefile_version() -> u32 {
        1
    }

    pub fn restore_node(&self) -> Box<dyn MyNode> {
        NodeType::from_str(self.type_.as_str());

        let mut node: Box<dyn MyNode> = NodeType::from_str(self.type_.as_str())
            .unwrap_or_default()
            .new_node();
        node.set_id(self.id.clone());
        node.set_xy(self.x, self.y);
        return node;
    }
}
