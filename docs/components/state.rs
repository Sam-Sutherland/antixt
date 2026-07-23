use antixt::{RequestFinished, RequestLifecycle};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SiteMetadata {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Default)]
pub struct RequestMetrics {
    completed: AtomicU64,
}

impl RequestMetrics {
    pub fn completed(&self) -> u64 {
        self.completed.load(Ordering::Relaxed)
    }
}

impl RequestLifecycle for RequestMetrics {
    fn finished(&self, _request: &RequestFinished<'_>) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }
}
