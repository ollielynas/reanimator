use imgui::{sys::{igSetNextWindowSize, ImVec2}, Style};
use savefile;
use project::Project;
use support::create_context;
#[macro_use]
extern crate savefile_derive;


mod project;
mod node;
mod nodes;
mod support;

fn main() {
    let mut project: Option<Project> = None;

    let mut ctx = create_context();
    
    Style::use_light_colors(ctx.style_mut());
    // ctx.load_ini_settings();
        support::simple_init(&mut ctx, file!(), move |_, ui| {
        
        if let Some(ref mut project) = project {
            project.render(ui);
            
        }else {
            // ReUi::load_and_apply(egui_ctx);
            
            project = Project::project_menu(ui);
            
        }
    });
}