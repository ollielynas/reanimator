use glium::glutin::surface::WindowSurface;
use glium::uniforms::SamplerBehavior;
use glium::{program, Display, Program, Surface, Texture2d};
use imgui::sys::{igSetNextWindowSize, ImGuiCond, ImGuiCond_Always, ImGuiCond_Once, ImVec2};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::{Renderer, Texture};
use imgui_winit_support::winit::dpi::LogicalSize;
use imgui_winit_support::winit::event::{Event, WindowEvent};
use imgui_winit_support::winit::event_loop::EventLoop;
use imgui_winit_support::winit::window::{Fullscreen, WindowBuilder};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;


mod clipboard;

pub const FONT_SIZE: f32 = 13.0;

// #[allow(dead_code)] // annoyingly, RA yells that this is unusued
// pub fn simple_init<F: FnMut(&mut bool, &mut Ui) + 'static>(imgui: &mut Context, title: &str, run_ui: F) {
//     init_with_startup(title, |_, _, _| {}, run_ui, None,  imgui);
// }
// #[allow(dead_code)] // annoyingly, RA yells that this is unusued
// pub fn simple_init_fullscreen<F: FnMut(&mut bool, &mut Ui) + 'static>(title: &str, run_ui: F,  imgui: &mut Context) {
//     init_with_startup(title, |_, _, _| {}, run_ui, Some(Fullscreen::Borderless(None)),  imgui);
// }

pub fn init_with_startup<FInit, FUi>(title: &str, mut startup: FInit, mut run_ui: FUi, fullscreen: Option<Fullscreen>, imgui: &mut Context)
where
    FInit: FnMut(&mut Context, &mut Renderer, &Display<WindowSurface>) + 'static,
    FUi: FnMut(&mut bool, &mut Ui, &Display<WindowSurface>, &mut Renderer) + 'static,
{
    // let mut imgui = create_context();

    let title = match Path::new(&title).file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => title,
    };
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    // Program::
    let builder = WindowBuilder::new()
        .with_title(title)
        .with_fullscreen(fullscreen)
        .with_inner_size(LogicalSize::new(1024, 512));
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .set_window_builder(builder)
        .build(&event_loop);
    // program!()
    
    let mut renderer = Renderer::init(imgui, &display).expect("Failed to initialize renderer");
    
    let mut display_texture = Texture2d::empty(&display, 512, 512).unwrap();


    let texture_id: imgui::TextureId = renderer.textures().insert(Texture {
        texture: Rc::new(display_texture),
        sampler: SamplerBehavior::default(),
    });

    if let Some(backend) = clipboard::init() {
        imgui.set_clipboard_backend(backend);
    } else {
        eprintln!("Failed to initialize clipboard");
    }


    let mut platform = WinitPlatform::init(imgui);
    {
        let dpi_mode = if let Ok(factor) = std::env::var("IMGUI_EXAMPLE_FORCE_DPI_FACTOR") {
            // Allow forcing of HiDPI factor for debugging purposes
            match factor.parse::<f64>() {
                Ok(f) => HiDpiMode::Locked(f),
                Err(e) => panic!("Invalid scaling factor: {}", e),
            }
        } else {
            HiDpiMode::Default
        };
        // renderer.textures().insert(texture)
        platform.attach_window(imgui.io_mut(), &window, dpi_mode);
    }

    let mut last_frame = Instant::now();

    startup(imgui, &mut renderer, &display);
    
    event_loop
        .run(move |event, window_target| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::AboutToWait => {
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                
                // window.set_maximized(true);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let ui = imgui.frame();

                let mut run = true;
                run_ui(&mut run, ui, &display, &mut renderer);
                if !run {
                    window_target.exit();
                }

                let mut target = display.draw();
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                platform.prepare_render(ui, &window);
                
                let draw_data = imgui.render();
                

                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    display.resize((new_size.width, new_size.height));
                }
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => window_target.exit(),
            event => {
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
        })
        .expect("EventLoop error");
}

/// Creates the imgui context
pub fn create_context() -> imgui::Context {
    let mut imgui = Context::create();
    // Fixed font size. Note imgui_winit_support uses "logical
    // pixels", which are physical pixels scaled by the devices
    // scaling factor. Meaning, 13.0 pixels should look the same size
    // on two different screens, and thus we do not need to scale this
    // value (as the scaling is handled by winit)
    imgui.fonts().add_font(&[
        FontSource::TtfData {
            data: include_bytes!("resources/Roboto-Regular.ttf"),
            size_pixels: FONT_SIZE,
            config: Some(FontConfig {
                // As imgui-glium-renderer isn't gamma-correct with
                // it's font rendering, we apply an arbitrary
                // multiplier to make the font a bit "heavier". With
                // default imgui-glow-renderer this is unnecessary.
                rasterizer_multiply: 1.5,
                // Oversampling font helps improve text rendering at
                // expense of larger font atlas texture.
                oversample_h: 4,
                oversample_v: 4,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("resources/mplus-1p-regular.ttf"),
            size_pixels: FONT_SIZE,
            config: Some(FontConfig {
                // Oversampling font helps improve text rendering at
                // expense of larger font atlas texture.
                oversample_h: 4,
                oversample_v: 4,
                // Range of glyphs to rasterize
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
    ]);
    imgui.set_ini_filename(None);

    imgui
}
