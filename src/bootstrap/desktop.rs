use crate::logging::init as init_logging;
use crate::market::mock::MockMarketDataProvider;
use crate::market::mock_realtime::MockRealtimeMarketDataProvider;
use crate::presentation::desktop::app::{ApexTraderApp, RealtimeProviderFactory};
use std::sync::Arc;

const SIMULATION_INTERVAL_MS: i64 = 60_000;
const SIMULATION_TICKS_PER_CANDLE: u32 = 20;
const SIMULATION_VOLUME: f64 = 35.0;

pub fn run() -> eframe::Result {
    if let Err(error) = init_logging() {
        eprintln!("Advertencia: no se pudo inicializar logging: {}", error);
    }

    tracing::info!("Iniciando ApexTrader");

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    let market_data_provider = Box::new(
        MockMarketDataProvider::new(50_000.0)
            .with_interval_ms(SIMULATION_INTERVAL_MS)
            .with_base_volume(SIMULATION_VOLUME),
    );
    let realtime_provider_factory: RealtimeProviderFactory = Arc::new(|last_candle| {
        let seed_provider = MockMarketDataProvider::new(last_candle.close)
            .with_interval_ms(last_candle.close_time_ms - last_candle.open_time_ms)
            .with_base_volume(SIMULATION_VOLUME);
        let seed = seed_provider.seed_for_symbol(&last_candle.symbol);
        MockRealtimeMarketDataProvider::from_last_candle(
            last_candle,
            SIMULATION_TICKS_PER_CANDLE,
            SIMULATION_VOLUME,
            seed,
        )
        .map(|provider| Box::new(provider) as _)
    });

    eframe::run_native(
        "ApexTrader",
        native_options,
        Box::new(move |creation_context| {
            Ok(Box::new(ApexTraderApp::new(
                creation_context,
                market_data_provider,
                realtime_provider_factory,
            )))
        }),
    )
}
