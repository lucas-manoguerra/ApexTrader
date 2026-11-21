use tracing_subscriber::{EnvFilter, fmt};

/// Inicializa el sistema de logging con `tracing`.
///
/// Configura:
/// - `EnvFilter`: controla qué nivel de logs se muestra mediante la variable
///   de entorno `RUST_LOG` (por defecto `info`).
/// - `fmt`: formateo legible para consola.
pub fn init() -> Result<(), tracing_subscriber::filter::ParseError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("Sistema de logging inicializado");
    Ok(())
}
