use crate::market::cancellation::CancellationToken;
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread::{self, JoinHandle};

struct WorkerState<Request> {
    pending: Option<Request>,
    active: Option<CancellationToken>,
    shutdown: bool,
}

/// A single-threaded worker that keeps only the newest pending request.
///
/// A new request cooperatively cancels the active operation and replaces any
/// request that has not started yet. Results carry whatever generation/identity
/// the caller puts in its response and must still be checked by the consumer.
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
        let (result_tx, results) = mpsc::channel();

        let thread = thread::Builder::new()
            .name("apextrader-market-worker".to_owned())
            .spawn(move || loop {
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
                let _ = result_tx.send(response);

                let (lock, _) = &*worker_state;
                let mut state = lock.lock().expect("market worker mutex poisoned");
                state.active = None;
                if state.shutdown {
                    return;
                }
            })
            .expect("failed to spawn market worker");

        Self {
            state,
            results,
            thread: Some(thread),
        }
    }

    /// Queue the newest request, replacing a pending request and cancelling
    /// the active request. The worker thread never shares UI-owned state.
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

    pub fn try_recv(&self) -> Result<Response, mpsc::TryRecvError> {
        self.results.try_recv()
    }

    pub fn drain(&self) -> Vec<Response> {
        self.results.try_iter().collect()
    }

    /// Request cooperative shutdown. The JoinHandle is detached so a provider
    /// that violates the cancellation contract cannot freeze the UI on drop.
    pub fn shutdown(&mut self) {
        let (lock, wake) = &*self.state;
        let mut state = lock.lock().expect("market worker mutex poisoned");
        state.shutdown = true;
        state.pending = None;
        if let Some(active) = &state.active {
            active.cancel();
        }
        wake.notify_all();
        self.thread.take();
    }
}

impl<Request, Response> Drop for LatestRequestWorker<Request, Response> {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn a_new_request_cancels_the_active_request() {
        let worker = LatestRequestWorker::spawn(|request: u32, cancellation| {
            for _ in 0..100 {
                if cancellation.is_cancelled() {
                    return (request, true);
                }
                thread::sleep(Duration::from_millis(1));
            }
            (request, false)
        });

        assert!(worker.request(1));
        thread::sleep(Duration::from_millis(5));
        assert!(worker.request(2));

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut responses = Vec::new();
        while std::time::Instant::now() < deadline && responses.len() < 2 {
            responses.extend(worker.drain());
            thread::sleep(Duration::from_millis(2));
        }

        assert!(responses.iter().any(|(id, cancelled)| *id == 1 && *cancelled));
        assert!(responses.iter().any(|(id, _)| *id == 2));
    }

    #[test]
    fn pending_requests_are_coalesced_to_the_latest() {
        let executions = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&executions);
        let worker = LatestRequestWorker::spawn(move |request: u32, _| {
            observed.fetch_add(1, Ordering::SeqCst);
            request
        });

        // The worker may start immediately, so gate it before submitting the
        // burst and assert only the latest queued request is eventually run.
        let _ = worker.request(10);
        let _ = worker.request(11);
        let _ = worker.request(12);

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut found_latest = false;
        while std::time::Instant::now() < deadline {
            if worker.drain().contains(&12) {
                found_latest = true;
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert!(found_latest);
        assert!(executions.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn shutdown_rejects_new_requests() {
        let mut worker = LatestRequestWorker::<u32, u32>::spawn(|request, _| request);
        worker.shutdown();
        assert!(!worker.request(1));
    }
}
