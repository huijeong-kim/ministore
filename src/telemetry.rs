use tracing::metadata::LevelFilter;

pub fn init_tracing(level: &str) -> Result<(), String> {
    let ministore_level = format!("ministore={}", level);

    let module_filter = tracing_subscriber::filter::EnvFilter::default()
        .add_directive(ministore_level.parse().unwrap())
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("h2=error".parse().unwrap());

    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_file(false)
        .with_thread_ids(true)
        .with_line_number(false)
        .with_target(true)
        .with_env_filter(module_filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|e| e.to_string())?;
    tracing::info!("Tracing initialized ({:?})..", LevelFilter::current());

    Ok(())
}
