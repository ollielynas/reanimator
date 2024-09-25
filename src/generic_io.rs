#[derive(Debug, PartialEq, Eq)]
pub enum EditTab {
    Nodes,
    BatchFileEdit,
    ProjectRes,
}

#[derive(Savefile)]
pub struct GenericIO {
    pub input_id: Option<String>,
    pub output_id: Option<String>,
}

impl Default for GenericIO {
    fn default() -> Self {
        GenericIO {
            input_id: None,
            output_id: None,
        }
    }
}
