use std::{fs, time::SystemTime};

use fast_smaz::Smaz;

use crate::LOG_TEXT;



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
        .chain(shared_dispatch.clone())
        .apply()?;

    Ok(())
}