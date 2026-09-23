use super::samples::SamplePoint;
use serde::{Deserialize, Serialize};

pub const MAX_IPC_REFRESH_HZ: u32 = 4;
pub const MIN_BATCH_INTERVAL_MS: u64 = 1000 / MAX_IPC_REFRESH_HZ as u64; // 250 ms

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleBatch {
    pub session_id: String,
    pub direction: String,
    pub samples: Vec<SamplePoint>,
    pub latest_bps: u64,
    pub latest_cpu_percent: Option<f64>,
    pub has_gaps: bool,
}

pub struct SampleBatcher {
    session_id: String,
    direction: String,
    pending: Vec<SamplePoint>,
    last_flush_t_ms: u64,
}

impl SampleBatcher {
    pub fn new(session_id: impl Into<String>, direction: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            direction: direction.into(),
            pending: Vec::new(),
            last_flush_t_ms: 0,
        }
    }

    pub fn set_direction(&mut self, direction: impl Into<String>) {
        self.direction = direction.into();
    }

    pub fn push(&mut self, sample: SamplePoint) -> Option<SampleBatch> {
        let t_ms = sample.t_ms;
        self.pending.push(sample);

        if t_ms.saturating_sub(self.last_flush_t_ms) >= MIN_BATCH_INTERVAL_MS {
            self.flush(t_ms)
        } else {
            None
        }
    }

    pub fn flush(&mut self, current_t_ms: u64) -> Option<SampleBatch> {
        if self.pending.is_empty() {
            return None;
        }

        let samples: Vec<SamplePoint> = std::mem::take(&mut self.pending);
        let latest = samples.last()?;
        let latest_bps = latest.bps;
        let latest_cpu_percent = latest.cpu_percent;
        let has_gaps = samples.iter().any(|s| s.gap);

        self.last_flush_t_ms = current_t_ms;

        Some(SampleBatch {
            session_id: self.session_id.clone(),
            direction: self.direction.clone(),
            samples,
            latest_bps,
            latest_cpu_percent,
            has_gaps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_batching_rate_limit() {
        let mut batcher = SampleBatcher::new("sess-1", "forward");

        // Sample at 100 ms (less than MIN_BATCH_INTERVAL_MS of 250ms)
        let s1 = SamplePoint {
            t_ms: 100,
            direction: "forward".into(),
            bps: 500_000_000,
            cpu_percent: Some(10.0),
            gap: false,
        };
        assert!(batcher.push(s1).is_none());

        // Sample at 260 ms (>= 250 ms from 0)
        let s2 = SamplePoint {
            t_ms: 260,
            direction: "forward".into(),
            bps: 550_000_000,
            cpu_percent: Some(12.0),
            gap: false,
        };
        let batch = batcher.push(s2).expect("debe generar un lote");
        assert_eq!(batch.samples.len(), 2);
        assert_eq!(batch.latest_bps, 550_000_000);
        assert!(!batch.has_gaps);
    }
}
