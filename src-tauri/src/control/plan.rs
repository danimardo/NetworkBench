pub use crate::model::plan::*;

/// Deriva el número recomendado de streams TCP según la velocidad de enlace en Mbit/s (Historias.md §11.3):
/// - Enlaces <= 100 Mbit/s: 1 stream
/// - Enlaces 100..1000 Mbit/s (1 GbE): 1 stream estándar (hasta 4 para multi-cola)
/// - Enlaces 2.5 GbE / 5 GbE: 4 streams
/// - Enlaces 10 GbE o superiores: 8 streams
///
/// Si no está disponible la velocidad de enlace, usa 1 stream por defecto.
pub fn derive_standard_streams(link_speed_mbps: Option<u64>) -> u32 {
    match link_speed_mbps {
        Some(speed) if speed >= 10_000 => 8,
        Some(speed) if speed >= 2_500 => 4,
        _ => 1,
    }
}

/// Deriva el tamaño de buffer en bytes para TCP:
/// - 64 KiB (65536) por defecto.
/// - 128 KiB (131072) para 10 GbE+.
pub fn derive_standard_buffer_size(link_speed_mbps: Option<u64>) -> u64 {
    match link_speed_mbps {
        Some(speed) if speed >= 10_000 => 131_072,
        _ => 65_536,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_streams_and_buffers() {
        assert_eq!(derive_standard_streams(None), 1);
        assert_eq!(derive_standard_streams(Some(100)), 1);
        assert_eq!(derive_standard_streams(Some(1000)), 1);
        assert_eq!(derive_standard_streams(Some(2500)), 4);
        assert_eq!(derive_standard_streams(Some(10000)), 8);

        assert_eq!(derive_standard_buffer_size(None), 65536);
        assert_eq!(derive_standard_buffer_size(Some(1000)), 65536);
        assert_eq!(derive_standard_buffer_size(Some(10000)), 131072);
    }
}
