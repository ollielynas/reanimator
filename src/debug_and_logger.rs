use std::{fs, thread, time::{Duration, SystemTime}};

use fast_smaz::Smaz;
use imgui::Ui;
use perf_monitor::{fd::fd_count_cur, io::get_process_io_stats, mem::get_process_memory_info};

use crate::LOG_TEXT;
use numfmt::*;



pub fn set_logger_mine() -> anyhow::Result<()> {

    fs::remove_file("output.log");


    let shared_dispatch = fern::Dispatch::new().into_shared();

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            if let Ok(ref mut log) = LOG_TEXT.lock() {
                let text = format!(
                    "\n[{} {}:{}:0 {}] {}",
                    record.level(),
                    record.file().unwrap_or_default(),
                    record
                        .line()
                        .unwrap_or(999999)
                        .to_string()
                        .replace("999999", ""),
                    humantime::format_rfc3339(SystemTime::now()),
                    message
                );
                match log.last_mut() {
                    Some(a) if a.len() < 500 => {
                        *a = [
                            a.smaz_decompress().unwrap_or_default(),
                            text.as_bytes().to_vec(),
                        ]
                        .concat()
                        .smaz_compress();
                    }
                    _ => log.push(text.smaz_compress()),
                }
            }
            out.finish(format_args!(
                "[{} {}:{}:0 {}] {}",
                record.level(),
                record.file().unwrap_or_default(),
                record
                    .line()
                    .unwrap_or(999999)
                    .to_string()
                    .replace("999999", ""),
                humantime::format_rfc3339(SystemTime::now()),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        // Apply globally
        .chain(shared_dispatch)
        .apply()?;

    Ok(())
}




pub fn profile(ui: &Ui, usage_p: f64, usage_t: f64) {
    let mut f = Formatter::new()
                .scales(Scales::binary())
                .precision(Precision::Significance(3))
                .suffix("B").unwrap();
    ui.window("debug profile")
    .no_decoration()
    .position([0.0,0.0], imgui::Condition::Always)
    .bg_alpha(0.7)
    .bring_to_front_on_focus(true)
    .focus_on_appearing(true)
    .mouse_inputs(false)
    .build(|| {
        // let load_average = sys_info::loadavg().unwrap_or(LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 });
        ui.text(format!("delta time: {}", ui.io().delta_time));
        ui.text(format!("fps: {}", ui.io().framerate));
        ui.text(format!("Active allocations: {}", ui.io().metrics_active_allocations));

        ui.text(format!("[CPU] process usage: {:.2}%, current thread usage: {:.2}%", usage_p, usage_t));

    // mem
    let mem_info = get_process_memory_info().unwrap();

    // This is so cursed, there must be a better way
    let num1 = f.fmt2(mem_info.resident_set_size as f64);
    let mut f = Formatter::new()
    .scales(Scales::binary())
    .precision(Precision::Significance(3))
    .suffix("B").unwrap();
    let num2 = f.fmt2(mem_info.virtual_memory_size as f64);
    ui.text(format!("[Memory] memory used: {} bytes, virtural memory used: {} bytes ", num1, num2));

    // fd
    let fd_num = fd_count_cur().unwrap();
    ui.text(format!("[FD] fd number: {}", fd_num));

    // io
    let io_stat = get_process_io_stats().unwrap();   
    ui.text(format!("[IO] io-in: {} bytes, io-out: {} bytes", io_stat.read_bytes, io_stat.write_bytes));
    });
}