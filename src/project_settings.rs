

#[derive(Savefile)]
pub struct ProjectSettings {
    pub render_ticker: bool,
}


impl Default for ProjectSettings {
    fn default() -> Self {
        ProjectSettings {
            render_ticker: false,
        }
    }
}