use serde::{Deserialize, Serialize};

pub const SAMPLE_INTERVAL_MS: u64 = 500;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplePoint {
    pub t_ms: u64,
    pub direction: String,
    pub bps: u64,
    pub cpu_percent: Option<f64>,
    pub gap: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SampleCollector {
    samples: Vec<SamplePoint>,
    last_sample_t_ms: Option<u64>,
}

impl SampleCollector {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            last_sample_t_ms: None,
        }
    }

    /// Añade una observación real medida
    pub fn record_sample(
        &mut self,
        t_ms: u64,
        direction: &str,
        bps: u64,
        cpu_percent: Option<f64>,
    ) {
        // Detectar si hubo un hueco temporal superior a 1.5x el intervalo (p. ej. > 750 ms)
        if let Some(last_t) = self.last_sample_t_ms {
            if t_ms.saturating_sub(last_t) > SAMPLE_INTERVAL_MS + 250 {
                // Registrar hueco explícito sin inventar valores
                self.samples.push(SamplePoint {
                    t_ms: last_t + SAMPLE_INTERVAL_MS,
                    direction: direction.to_string(),
                    bps: 0,
                    cpu_percent: None,
                    gap: true,
                });
            }
        }

        self.samples.push(SamplePoint {
            t_ms,
            direction: direction.to_string(),
            bps,
            cpu_percent,
            gap: false,
        });

        self.last_sample_t_ms = Some(t_ms);
    }

    /// Registra un hueco de forma deliberada ante desconexión o pérdida temporal
    pub fn record_explicit_gap(&mut self, t_ms: u64, direction: &str) {
        self.samples.push(SamplePoint {
            t_ms,
            direction: direction.to_string(),
            bps: 0,
            cpu_percent: None,
            gap: true,
        });
        self.last_sample_t_ms = Some(t_ms);
    }

    pub fn samples(&self) -> &[SamplePoint] {
        &self.samples
    }

    pub fn into_samples(self) -> Vec<SamplePoint> {
        self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_recording_and_gap_detection() {
        let mut collector = SampleCollector::new();

        collector.record_sample(0, "forward", 940_000_000, Some(3.2));
        collector.record_sample(500, "forward", 950_000_000, Some(3.5));

        // Salto de 1500 ms (debe insertar hueco explícito)
        collector.record_sample(2000, "forward", 945_000_000, Some(3.6));

        let list = collector.samples();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].t_ms, 0);
        assert!(!list[0].gap);
        assert_eq!(list[1].t_ms, 500);
        assert!(!list[1].gap);

        // Muestra de hueco
        assert_eq!(list[2].t_ms, 1000);
        assert!(list[2].gap);
        assert_eq!(list[2].bps, 0);

        assert_eq!(list[3].t_ms, 2000);
        assert!(!list[3].gap);
    }
}
