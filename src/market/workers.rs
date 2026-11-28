use crate::market::cancellation::CancellationToken;
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread::{self, JoinHandle};

struct WorkerState<Request> {
    pending: Option<Request>,
    active: Option<CancellationToken>,
    shutdown: bool,
}
pub struct LatestRequestWorker<Request, Response> {
    state: Arc<(Mutex<WorkerState<Request>>, Condvar)>,
    results: mpsc::Receiver<Response>,
    thread: Option<JoinHandle<()>>,
}

impl<Request, Response> LatestRequestWorker<Request, Response>
where
    Request: Send + 'static,
    Response: Send + 'static,
{
    pub fn spawn<Execute>(mut execute: Execute) -> Self
    where
        Execute: FnMut(Request, CancellationToken) -> Response + Send + 'static,
    {
        let state = Arc::new((
            Mutex::new(WorkerState {
                pending: None,
                active: None,
                shutdown: false,
            }),
            Condvar::new(),
        ));
        let worker_state = Arc::clone(&state);
        let (result_tx, results) = mpsc::sync_channel(1);

        let thread = thread::Builder::new()
            .name("apextrader-market-worker".to_owned())
            .spawn(move || {
                loop {
                    let request = {
                        let (lock, wake) = &*worker_state;
                        let mut state = lock.lock().expect("market worker mutex poisoned");
                        while state.pending.is_none() && !state.shutdown {
                            state = wake.wait(state).expect("market worker mutex poisoned");
                        }
                        if state.shutdown {
                            return;
                        }

                        let request = state.pending.take().expect("pending request checked");
                        let cancellation = CancellationToken::new();
                        state.active = Some(cancellation.clone());
                        (request, cancellation)
                    };

                    let (request, cancellation) = request;
                    let response = execute(request, cancellation.clone());
                    let shutting_down = {
                        let (lock, _) = &*worker_state;
                        let mut state = lock.lock().expect("market worker mutex poisoned");
                        state.active = None;
                        state.shutdown
                    };
                    if shutting_down {
                        return;
                    }

                    let mut pending_response = Some(response);
                    while let Some(response) = pending_response.take() {
                        match result_tx.try_send(response) {
                            Ok(()) => break,
                            Err(mpsc::TrySendError::Full(response)) => {
                                let shutting_down = {
                                    let (lock, _) = &*worker_state;
                                    lock.lock()
                                        .expect("market worker mutex poisoned")
                                        .shutdown
                                };
                                if shutting_down {
                                    return;
                                }
                                pending_response = Some(response);
                                thread::sleep(Duration::from_millis(5));
                            }
                            Err(mpsc::TrySendError::Disconnected(_)) => return,
                        }
                    }
                }
            })
            .expect("failed to spawn market worker");

        Self {
            state,
            results,
            thread: Some(thread),
        }
    }
    pub fn request(&self, request: Request) -> bool {
        let (lock, wake) = &*self.state;
        let mut state = lock.lock().expect("market worker mutex poisoned");
        if state.shutdown {
            return false;
        }
        if let Some(active) = &state.active {
            active.cancel();
        }
        state.pending = Some(request);
        wake.notify_one();
        true
    }

    pub fn cancel(&self) {
        let (lock, _) = &*self.state;
        let mut state = lock.lock().expect("market worker mutex poisoned");
        state.pending = None;
        if let Some(active) = &state.active {
            active.cancel();
        }
    }

    pub fn try_recv(&self) -> Result<Response, mpsc::TryRecvError> {
        self.results.try_recv()
    }

    pub fn drain(&self) -> Vec<Response> {
        self.results.try_iter().collect()
    }
    pub fn shutdown(&mut self) {
        let (lock, wake) = &*self.state;
        let mut state = lock.lock().expect("market worker mutex poisoned");
        state.shutdown = true;
        state.pending = None;
        if let Some(active) = &state.active {
            active.cancel();
        }
        wake.notify_all();
        let thread = self.thread.take();
        drop(state);
        if let Some(thread) = thread {
            let _ = thread.join();
        }
    }
}

impl<Request, Response> Drop for LatestRequestWorker<Request, Response> {
    fn drop(&mut self) {
        let (lock, wake) = &*self.state;
        let mut state = lock.lock().expect("market worker mutex poisoned");
        state.shutdown = true;
        state.pending = None;
        if let Some(active) = &state.active {
            active.cancel();
        }
        wake.notify_all();
        let thread = self.thread.take();
        drop(state);
        if let Some(thread) = thread {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn a_new_request_cancels_the_active_request() {
        let (started_tx, started_rx) = mpsc::channel();
        let worker = LatestRequestWorker::spawn(move |request: u32, cancellation| {
            let _ = started_tx.send(request);
            for _ in 0..100 {
                if cancellation.is_cancelled() {
                    return (request, true);
                }
                thread::sleep(Duration::from_millis(1));
            }
            (request, false)
        });

        assert!(worker.request(1));
        assert_eq!(started_rx.recv_timeout(Duration::from_secs(1)), Ok(1));
        assert!(worker.request(2));

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut responses = Vec::new();
        while std::time::Instant::now() < deadline && responses.len() < 2 {
            responses.extend(worker.drain());
            thread::sleep(Duration::from_millis(2));
        }

        assert!(
            responses
                .iter()
                .any(|(id, cancelled)| *id == 1 && *cancelled)
        );
        assert!(responses.iter().any(|(id, _)| *id == 2));
    }

    #[test]
    fn pending_requests_are_coalesced_to_the_latest() {
        let executions = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&executions);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = LatestRequestWorker::spawn(move |request: u32, _| {
            observed.fetch_add(1, Ordering::SeqCst);
            if request == 10 {
                let _ = started_tx.send(());
                let _ = release_rx.recv();
            }
            request
        });

        assert!(worker.request(10));
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("first request started");
        assert!(worker.request(11));
        assert!(worker.request(12));
        release_tx.send(()).expect("release worker");

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut responses = Vec::new();
        while std::time::Instant::now() < deadline && !responses.contains(&12) {
            responses.extend(worker.drain());
            thread::sleep(Duration::from_millis(2));
        }
        assert!(responses.contains(&10));
        assert!(responses.contains(&12));
        assert!(!responses.contains(&11));
        assert_eq!(executions.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn shutdown_cancels_and_joins_an_active_request() {
        let (started_tx, started_rx) = mpsc::channel();
        let worker = LatestRequestWorker::spawn(move |request: u32, cancellation| {
            let _ = started_tx.send(());
            while !cancellation.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
            request
        });

        assert!(worker.request(1));
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("request started");
        drop(worker);
    }

    #[test]
    fn shutdown_rejects_new_requests() {
        let mut worker = LatestRequestWorker::<u32, u32>::spawn(|request, _| request);
        worker.shutdown();
        assert!(!worker.request(1));
    }
}

use crate::market::errors::MarketError;
use crate::market::provider::{RealtimeMarketDataProvider, RealtimeMarketUpdate};
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug)]
pub struct RealtimeBatch {
    pub updates: Vec<RealtimeMarketUpdate>,
    pub latest_error: Option<MarketError>,
}

struct RealtimeQueue {
    updates: VecDeque<RealtimeMarketUpdate>,
    closed_keys: VecDeque<(crate::market::symbol::Symbol, i64)>,
    latest_error: Option<MarketError>,
    closed: bool,
}
#[derive(Clone)]
pub struct RealtimeMailbox {
    inner: Arc<(Mutex<RealtimeQueue>, Condvar)>,
    capacity: usize,
}

impl RealtimeMailbox {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "realtime mailbox capacity must be positive");
        Self {
            inner: Arc::new((
                Mutex::new(RealtimeQueue {
                    updates: VecDeque::new(),
                    closed_keys: VecDeque::new(),
                    latest_error: None,
                    closed: false,
                }),
                Condvar::new(),
            )),
            capacity,
        }
    }

    fn publish(&self, update: RealtimeMarketUpdate, cancellation: &CancellationToken) -> bool {
        let (lock, wake) = &*self.inner;
        let mut queue = lock.lock().expect("realtime mailbox mutex poisoned");

        if update.is_closed {
            if let Some(existing) = queue.updates.iter_mut().find(|existing| {
                existing.is_closed
                    && existing.candle.symbol == update.candle.symbol
                    && existing.candle.open_time_ms == update.candle.open_time_ms
            }) {
                *existing = update;
                wake.notify_all();
                return true;
            }

            while queue.updates.len() >= self.capacity
                && !queue.closed
                && !cancellation.is_cancelled()
            {
                let (next, _) = wake
                    .wait_timeout(queue, Duration::from_millis(20))
                    .expect("realtime mailbox mutex poisoned");
                queue = next;
            }
            if queue.closed || cancellation.is_cancelled() {
                return false;
            }

            let closed_symbol = update.candle.symbol.clone();
            let closed_time = update.candle.open_time_ms;
            queue.updates.retain(|queued| {
                queued.is_closed
                    || queued.candle.symbol != closed_symbol
                    || queued.candle.open_time_ms != closed_time
            });
            queue.closed_keys.push_back((closed_symbol, closed_time));
            while queue.closed_keys.len() > self.capacity.saturating_mul(2) {
                let _ = queue.closed_keys.pop_front();
            }
            queue.updates.push_back(update);
            wake.notify_all();
            return true;
        }

        if queue.closed || cancellation.is_cancelled() {
            return false;
        }

        if queue.closed_keys.iter().any(|(symbol, time)| {
            *symbol == update.candle.symbol && *time == update.candle.open_time_ms
        }) {
            return true;
        }

        if let Some(existing) = queue.updates.iter_mut().find(|existing| {
            !existing.is_closed
                && existing.candle.symbol == update.candle.symbol
                && existing.candle.open_time_ms == update.candle.open_time_ms
        }) {
            *existing = update;
            wake.notify_all();
            return true;
        }

        if queue.updates.len() >= self.capacity {
            if let Some(open_position) = queue.updates.iter().position(|item| !item.is_closed) {
                let _ = queue.updates.remove(open_position);
            } else {
                return true;
            }
        }

        queue.updates.push_back(update);
        wake.notify_all();
        true
    }

    fn publish_error(&self, error: MarketError) {
        let (lock, wake) = &*self.inner;
        let mut queue = lock.lock().expect("realtime mailbox mutex poisoned");
        if !queue.closed {
            queue.latest_error = Some(error);
            wake.notify_all();
        }
    }

    pub fn drain(&self) -> RealtimeBatch {
        let (lock, wake) = &*self.inner;
        let mut queue = lock.lock().expect("realtime mailbox mutex poisoned");
        let updates = queue.updates.drain(..).collect();
        let latest_error = queue.latest_error.take();
        wake.notify_all();
        RealtimeBatch {
            updates,
            latest_error,
        }
    }

    fn close(&self) {
        let (lock, wake) = &*self.inner;
        let mut queue = lock.lock().expect("realtime mailbox mutex poisoned");
        queue.closed = true;
        wake.notify_all();
    }
}
pub struct RealtimeWorker {
    cancellation: CancellationToken,
    mailbox: RealtimeMailbox,
    thread: Option<JoinHandle<()>>,
}

impl RealtimeWorker {
    pub fn spawn(
        provider: Box<dyn RealtimeMarketDataProvider>,
        interval: Duration,
        capacity: usize,
    ) -> Self {
        Self::spawn_with_factory(move || Ok(provider), interval, capacity)
    }

    pub fn spawn_with_factory<Initialize>(
        initialize: Initialize,
        interval: Duration,
        capacity: usize,
    ) -> Self
    where
        Initialize:
            FnOnce() -> Result<Box<dyn RealtimeMarketDataProvider>, MarketError> + Send + 'static,
    {
        let cancellation = CancellationToken::new();
        let worker_cancellation = cancellation.clone();
        let mailbox = RealtimeMailbox::new(capacity);
        let worker_mailbox = mailbox.clone();

        let thread = thread::Builder::new()
            .name("apextrader-realtime-worker".to_owned())
            .spawn(move || {
                let mut provider = match initialize() {
                    Ok(provider) => provider,
                    Err(error) => {
                        worker_mailbox.publish_error(error);
                        return;
                    }
                };

                while !worker_cancellation.is_cancelled() {
                    match provider.next_update_cancellable(&worker_cancellation) {
                        Ok(update) => {
                            if !worker_mailbox.publish(update, &worker_cancellation) {
                                break;
                            }
                        }
                        Err(MarketError::Cancelled) => break,
                        Err(error) => worker_mailbox.publish_error(error),
                    }

                    if worker_cancellation.is_cancelled() {
                        break;
                    }
                    thread::sleep(interval);
                }
            })
            .expect("failed to spawn realtime worker");

        Self {
            cancellation,
            mailbox,
            thread: Some(thread),
        }
    }

    pub fn drain(&self) -> RealtimeBatch {
        self.mailbox.drain()
    }

    pub fn shutdown(&mut self) {
        self.cancellation.cancel();
        self.mailbox.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for RealtimeWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod realtime_tests {
    use super::*;
    use crate::market::candle::MarketCandle;
    use crate::market::symbol::Symbol;
    use std::str::FromStr;

    fn update(open_time_ms: i64, is_closed: bool, close: f64) -> RealtimeMarketUpdate {
        RealtimeMarketUpdate {
            candle: MarketCandle {
                symbol: Symbol::from_str("BTC/USDT").expect("valid symbol"),
                open_time_ms,
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close,
                volume: 10.0,
                close_time_ms: open_time_ms + 60_000,
            },
            is_closed,
        }
    }

    #[test]
    fn open_candle_updates_are_coalesced_by_identity() {
        let mailbox = RealtimeMailbox::new(2);
        let cancellation = CancellationToken::new();

        assert!(mailbox.publish(update(0, false, 101.0), &cancellation));
        assert!(mailbox.publish(update(0, false, 102.0), &cancellation));

        let batch = mailbox.drain();
        assert_eq!(batch.updates.len(), 1);
        assert_eq!(batch.updates[0].candle.close, 102.0);
    }

    #[test]
    fn open_updates_never_evict_closed_candles() {
        let mailbox = RealtimeMailbox::new(2);
        let cancellation = CancellationToken::new();

        assert!(mailbox.publish(update(0, true, 101.0), &cancellation));
        assert!(mailbox.publish(update(60_000, true, 102.0), &cancellation));
        assert!(mailbox.publish(update(120_000, false, 103.0), &cancellation));

        let batch = mailbox.drain();
        assert_eq!(batch.updates.len(), 2);
        assert!(batch.updates.iter().all(|item| item.is_closed));
    }

    #[test]
    fn closed_updates_wait_for_capacity_instead_of_being_dropped() {
        let mailbox = RealtimeMailbox::new(1);
        let cancellation = CancellationToken::new();
        assert!(mailbox.publish(update(0, true, 101.0), &cancellation));

        let worker_mailbox = mailbox.clone();
        let worker_cancellation = cancellation.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let _ = started_tx.send(());
            let result = worker_mailbox.publish(update(60_000, true, 102.0), &worker_cancellation);
            let _ = done_tx.send(result);
        });

        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("publisher started");
        assert!(done_rx.recv_timeout(Duration::from_millis(30)).is_err());

        let first_batch = mailbox.drain();
        assert_eq!(first_batch.updates.len(), 1);
        assert_eq!(done_rx.recv_timeout(Duration::from_secs(1)), Ok(true));
        worker.join().expect("publisher thread joined");

        let second_batch = mailbox.drain();
        assert_eq!(second_batch.updates.len(), 1);
        assert_eq!(second_batch.updates[0].candle.open_time_ms, 60_000);
    }

    #[test]
    fn late_open_update_cannot_reopen_a_closed_candle() {
        let mailbox = RealtimeMailbox::new(2);
        let cancellation = CancellationToken::new();

        assert!(mailbox.publish(update(0, true, 101.0), &cancellation));
        let closed = mailbox.drain();
        assert_eq!(closed.updates.len(), 1);
        assert!(closed.updates[0].is_closed);

        assert!(mailbox.publish(update(0, false, 99.0), &cancellation));
        assert!(mailbox.drain().updates.is_empty());
    }

    #[test]
    fn mailbox_reports_the_latest_provider_error() {
        let mailbox = RealtimeMailbox::new(2);
        mailbox.publish_error(MarketError::Network("first".to_owned()));
        mailbox.publish_error(MarketError::Network("latest".to_owned()));

        let batch = mailbox.drain();
        assert_eq!(
            batch.latest_error,
            Some(MarketError::Network("latest".to_owned()))
        );
    }
}
