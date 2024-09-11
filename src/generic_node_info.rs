use crate::{node::MyNode, nodes::node_enum::NodeType};



#[derive(Savefile)]
pub struct GenericNodeInfo {
    pub x: f32,
    pub y: f32,
    pub type_: NodeType,
    pub id: String,
}


impl GenericNodeInfo {
    pub fn savefile_version() -> u32 {0}


    pub fn restore_node(&self) -> Box<dyn MyNode> {
        let mut node: Box<dyn MyNode> = self.type_.new_node();
        node.set_id(self.id.clone());
        node.set_xy(self.x, self.y);
        return node;
    }
}
