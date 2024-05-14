
pub fn setup_logging() -> Result<(), fern::InitError> {

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file("rrdp.log")?)
        .chain(std::io::stdout())
        // .chain(
        //     fern::Dispatch::new()
        //         .level(log::LevelFilter::Error)
        //         .chain(fern::Panic))
        .apply().ok();


    Ok(())
}

