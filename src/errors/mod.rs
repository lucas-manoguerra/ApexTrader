pub mod candle;
pub mod chart;
pub mod price;
pub mod quantity;
pub mod renderer;

pub use candle::CandleError;
pub use chart::ChartError;
pub use price::PriceError;
pub use quantity::QuantityError;
pub use renderer::RendererError;

/// Error general de la aplicación.
#[derive(Debug, thiserror::Error)]
pub enum ApexTraderError {
    #[error("Chart error: {0}")]
    Chart(#[from] ChartError),

    #[error("Renderer error: {0}")]
    Renderer(#[from] RendererError),
}
