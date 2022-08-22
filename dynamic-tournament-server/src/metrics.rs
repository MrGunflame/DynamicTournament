use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct Metrics {
    pub http_requests_total: Counter,

    pub websocket_connections_total: Counter,
    pub websocket_connections_current: Gauge,
}

macro_rules! impl_serialize {
    ($this:expr, $($metric:ident),*$(,)?) => {
        let mut capacity = 0;

        $(
            let $metric = $this.$metric.0.load(Ordering::Relaxed);

            // Two extra bytes for space and '\n'.
            capacity += stringify!($metric).len() + 2;

            capacity += ((($metric as f32).log10().floor() + 1.0) as usize).max(1);
        )*

        let mut buf = Vec::with_capacity(capacity);

        $(
            let _ = writeln!(buf, "{} {}", stringify!($metric), $metric);
        )*

        debug_assert_eq!(buf.len(), capacity);

        buf
    };
}

impl Metrics {
    pub fn serialize(&self) -> Vec<u8> {
        impl_serialize! {
            self,
            http_requests_total,
            websocket_connections_total,
            websocket_connections_current,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Counter(Arc<AtomicUsize>);

impl Counter {
    pub fn inc(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug, Default)]
pub struct Gauge(Arc<AtomicUsize>);

impl Gauge {
    pub fn inc(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec(&self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// A guard around a [`Gauge`]. The gauge is automatically incremented when created
/// and decremented when the guard is dropped.
#[derive(Debug)]
pub struct GaugeGuard {
    gauge: Gauge,
}

impl GaugeGuard {
    /// Creates a new `GaugeGuard`. This will increment the gauge.
    #[inline]
    pub fn new(gauge: Gauge) -> Self {
        gauge.inc();

        Self { gauge }
    }
}

impl Clone for GaugeGuard {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.gauge.clone())
    }
}

impl Drop for GaugeGuard {
    #[inline]
    fn drop(&mut self) {
        self.gauge.dec();
    }
}

#[cfg(test)]
mod tests {
    use super::Metrics;

    #[test]
    fn test_metrics_serialize() {
        let metrics = Metrics::default();
        metrics.serialize();
    }
}
