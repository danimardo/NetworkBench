use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkProtocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkDirection {
    #[serde(rename = "forward")]
    Forward,
    #[serde(rename = "reverse")]
    Reverse,
    #[serde(rename = "both_sequential", alias = "sequential")]
    BothSequential,
    #[serde(rename = "both", alias = "both_simultaneous", alias = "simultaneous")]
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkPlan {
    pub protocol: BenchmarkProtocol,
    pub direction: BenchmarkDirection,
    pub streams: u32,
    pub warmup_seconds: u32,
    pub measure_seconds: u32,
    pub cooldown_seconds: u32,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_size_bytes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub udp_target_rate_bps: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub udp_packet_size_bytes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_capacity_bps: Option<String>,
}

impl BenchmarkPlan {
    pub fn new_standard_tcp(port: u16) -> Self {
        Self {
            protocol: BenchmarkProtocol::Tcp,
            direction: BenchmarkDirection::Forward,
            streams: 1,
            warmup_seconds: 1,
            measure_seconds: 10,
            cooldown_seconds: 1,
            port,
            buffer_size_bytes: None,
            udp_target_rate_bps: None,
            udp_packet_size_bytes: None,
            expected_capacity_bps: None,
        }
    }

    pub fn is_simultaneous(&self) -> bool {
        matches!(self.direction, BenchmarkDirection::Both)
    }

    pub fn is_udp(&self) -> bool {
        self.protocol == BenchmarkProtocol::Udp
    }

    pub fn effective_udp_packet_size(&self) -> u32 {
        self.udp_packet_size_bytes.unwrap_or(1472)
    }

    pub fn validate(&self) -> Result<(), String> {
        let max_streams = match self.direction {
            BenchmarkDirection::Both => 32,
            _ => 64,
        };

        if self.streams == 0 || self.streams > max_streams {
            return Err(format!(
                "Número de streams inválido: {} (debe estar entre 1 y {})",
                self.streams, max_streams
            ));
        }

        if self.warmup_seconds > 10 {
            return Err("El tiempo de calentamiento no puede superar 10 s".to_string());
        }

        if self.measure_seconds < 5 || self.measure_seconds > 300 {
            return Err(format!(
                "El tiempo de medición debe estar entre 5 y 300 s (recibido: {})",
                self.measure_seconds
            ));
        }

        if self.cooldown_seconds > 10 {
            return Err("El tiempo de enfriamiento no puede superar 10 s".to_string());
        }

        if self.port < 1024 || self.port > 65000 {
            return Err(format!(
                "El puerto base debe estar entre 1024 y 65000 (recibido: {})",
                self.port
            ));
        }

        if let Some(ref buf_str) = self.buffer_size_bytes {
            let buf_val = buf_str
                .parse::<u64>()
                .map_err(|_| "El tamaño de buffer debe ser un número entero".to_string())?;
            if buf_val < 4096 || buf_val > 4_194_304 || !buf_val.is_power_of_two() {
                return Err(format!(
                    "El tamaño de buffer debe ser potencia de 2 entre 4 KB (4096) y 4 MB (4194304) (recibido: {})",
                    buf_val
                ));
            }
        }

        if let Some(ref target_str) = self.udp_target_rate_bps {
            let rate_val = target_str
                .parse::<u64>()
                .map_err(|_| "La tasa objetivo UDP debe ser un número entero de bit/s".to_string())?;
            if rate_val < 1_000_000 || rate_val > 100_000_000_000 {
                return Err(format!(
                    "La tasa objetivo UDP debe estar entre 1 Mbit/s y 100 000 Mbit/s (recibido: {} bit/s)",
                    rate_val
                ));
            }
        }

        if let Some(pkt_size) = self.udp_packet_size_bytes {
            if pkt_size < 64 || pkt_size > 65507 {
                return Err(format!(
                    "El tamaño de datagrama UDP debe estar entre 64 y 65 507 bytes (recibido: {})",
                    pkt_size
                ));
            }
        }

        if let Some(ref exp_str) = self.expected_capacity_bps {
            let cap_val = exp_str
                .parse::<u64>()
                .map_err(|_| "La capacidad esperada debe ser un número entero de bit/s".to_string())?;
            if cap_val < 1_000_000 || cap_val > 400_000_000_000 {
                return Err(format!(
                    "La capacidad esperada debe estar entre 1 Mbit/s y 400 000 Mbit/s (recibido: {} bit/s)",
                    cap_val
                ));
            }
        }

        Ok(())
    }
}
