
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("No se pudo crear el contexto de pintura")]
    PaintContextUnavailable,

    #[error("El rectángulo de dibujo está vacío")]
    EmptyDrawRect,

    #[error("Falló la inicialización del backend gráfico: {0}")]
    BackendInitFailed(String),
}
