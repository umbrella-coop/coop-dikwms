//! Retry and circuit-breaker primitives (SPEC-027 REQ-003).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Exponential backoff: base * 2^attempt, capped at `max`.
#[derive(Debug, Clone)]
pub struct Backoff {
    base: Duration,
    max: Duration,
    attempt: u32,
}

impl Backoff {
    pub fn new(base: Duration, max: Duration) -> Self {
        Self {
            base,
            max,
            attempt: 0,
        }
    }

    /// Next delay, then advances the attempt counter.
    pub fn next_delay(&mut self) -> Duration {
        let multiplier = 1u64.checked_shl(self.attempt).unwrap_or(u64::MAX);
        let delay = Duration::from_millis(
            (self.base.as_millis() as u64)
                .saturating_mul(multiplier)
                .min(self.max.as_millis() as u64),
        );
        self.attempt += 1;
        delay
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

/// Circuit breaker: halts when >=50% of events in a rolling window are
/// failures, once a minimum sample count is reached (SPEC-027 REQ-003).
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    window: Duration,
    min_samples: usize,
    events: VecDeque<(Instant, bool)>,
}

impl CircuitBreaker {
    pub fn new(window: Duration, min_samples: usize) -> Self {
        Self {
            window,
            min_samples,
            events: VecDeque::new(),
        }
    }

    fn prune(&mut self) {
        let cutoff = Instant::now() - self.window;
        while let Some(&(t, _)) = self.events.front() {
            if t < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn record_success(&mut self) {
        self.prune();
        self.events.push_back((Instant::now(), true));
    }

    pub fn record_failure(&mut self) {
        self.prune();
        self.events.push_back((Instant::now(), false));
    }

    /// Failure rate over the rolling window (prunes stale events).
    pub fn failure_rate(&mut self) -> f64 {
        self.prune();
        if self.events.is_empty() {
            return 0.0;
        }
        let failures = self.events.iter().filter(|(_, ok)| !ok).count();
        failures as f64 / self.events.len() as f64
    }

    /// True when enough samples exist and failure rate >= 0.5 — callers halt.
    pub fn should_halt(&mut self) -> bool {
        self.prune();
        if self.events.len() < self.min_samples {
            return false;
        }
        self.failure_rate() >= 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_and_caps() {
        let mut b = Backoff::new(Duration::from_millis(1), Duration::from_millis(5));
        assert_eq!(b.next_delay(), Duration::from_millis(1));
        assert_eq!(b.next_delay(), Duration::from_millis(2));
        assert_eq!(b.next_delay(), Duration::from_millis(4));
        assert_eq!(b.next_delay(), Duration::from_millis(5));
        assert_eq!(b.next_delay(), Duration::from_millis(5));
    }

    #[test]
    fn backoff_reset() {
        let mut b = Backoff::new(Duration::from_millis(1), Duration::from_millis(100));
        b.next_delay();
        b.next_delay();
        b.reset();
        assert_eq!(b.next_delay(), Duration::from_millis(1));
    }

    #[test]
    fn breaker_halts_at_half_failures() {
        let mut cb = CircuitBreaker::new(Duration::from_secs(60), 4);
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        cb.record_success();
        assert!(cb.should_halt());
        assert_eq!(cb.failure_rate(), 0.5);
    }

    #[test]
    fn breaker_needs_min_samples() {
        let mut cb = CircuitBreaker::new(Duration::from_secs(60), 4);
        cb.record_failure();
        cb.record_success();
        assert!(!cb.should_halt());
    }

    #[test]
    fn breaker_prunes_stale_events() {
        let mut cb = CircuitBreaker::new(Duration::from_millis(50), 1);
        cb.record_success();
        std::thread::sleep(Duration::from_millis(60));
        cb.record_failure();
        assert_eq!(cb.failure_rate(), 1.0);
        assert!(cb.should_halt());
    }
}
