use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn check(&self) -> Result<(), crate::market::errors::MarketError> {
        if self.is_cancelled() {
            Err(crate::market::errors::MarketError::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_cancellation_state() {
        let token = CancellationToken::new();
        let worker_token = token.clone();
        assert!(!worker_token.is_cancelled());

        token.cancel();

        assert!(worker_token.is_cancelled());
        assert!(matches!(
            worker_token.check(),
            Err(crate::market::errors::MarketError::Cancelled)
        ));
    }
}
