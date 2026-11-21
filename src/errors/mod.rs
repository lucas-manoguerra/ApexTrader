pub mod chart;
pub mod renderer;

pub use chart::ChartError;
pub use renderer::RendererError;

/// Error general de la aplicación.
#[derive(Debug, thiserror::Error)]
pub enum ApexTraderError {
    #[error("Chart error: {0}")]
    Chart(#[from] ChartError),

    #[error("Renderer error: {0}")]
    Renderer(#[from] RendererError),
}
