use crate::model::plan::{BenchmarkPlan, BenchmarkProtocol};
use std::net::{SocketAddr, TcpListener, UdpSocket};

pub const DEFAULT_CONTROL_PORT: u16 = 7411;
pub const DEFAULT_BENCHMARK_PORT: u16 = 7412;
pub const MAX_PORT_BLOCK_SIZE: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortAllocation {
    pub base_port: u16,
    pub count: u32,
    pub forward_range: (u16, u16),
    pub reverse_range: (u16, u16),
    pub is_simultaneous: bool,
}

impl PortAllocation {
    /// Comprueba que no hay solape entre los rangos de puertos emisor y receptor
    /// cuando la prueba es simultánea.
    pub fn has_overlap(&self) -> bool {
        if !self.is_simultaneous {
            return false;
        }
        let (f_start, f_end) = self.forward_range;
        let (r_start, r_end) = self.reverse_range;
        !(f_end < r_start || r_end < f_start)
    }
}

/// Comprueba si un puerto TCP está disponible para ser escuchado localmente.
pub fn is_port_available(port: u16) -> bool {
    is_port_available_for_protocol(port, BenchmarkProtocol::Tcp)
}

/// Comprueba si un puerto está disponible para un protocolo dado (TCP o UDP).
pub fn is_port_available_for_protocol(port: u16, protocol: BenchmarkProtocol) -> bool {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    match protocol {
        BenchmarkProtocol::Tcp => TcpListener::bind(addr).is_ok(),
        BenchmarkProtocol::Udp => UdpSocket::bind(addr).is_ok(),
    }
}

/// Comprueba si un rango de puertos TCP está libre.
pub fn check_port_range(base_port: u16, count: u32) -> bool {
    check_port_block(base_port, count, BenchmarkProtocol::Tcp)
}

/// Comprueba si un bloque de puertos contiguos está disponible para el protocolo dado.
pub fn check_port_block(base_port: u16, count: u32, protocol: BenchmarkProtocol) -> bool {
    if base_port < 1024 {
        return false;
    }
    let end_port = base_port as u32 + count.saturating_sub(1);
    if end_port > 65535 {
        return false;
    }
    for p in base_port..=(end_port as u16) {
        if !is_port_available_for_protocol(p, protocol) {
            return false;
        }
    }
    true
}

/// Encuentra el primer puerto TCP libre dentro del rango permitido (1024..65535).
pub fn find_available_port(preferred: u16) -> Result<u16, String> {
    find_available_port_block(preferred, 1, BenchmarkProtocol::Tcp)
}

/// Encuentra un bloque libre de `count` puertos contiguos para el protocolo dado,
/// comenzando en `preferred_base` y desplazándose en saltos seguros de 64 puertos.
pub fn find_available_port_block(
    preferred_base: u16,
    count: u32,
    protocol: BenchmarkProtocol,
) -> Result<u16, String> {
    if count == 0 || count > MAX_PORT_BLOCK_SIZE {
        return Err(format!(
            "Tamaño de bloque de puertos inválido: {} (debe estar entre 1 y {})",
            count, MAX_PORT_BLOCK_SIZE
        ));
    }

    let max_base = 65535u32.saturating_sub(count.saturating_sub(1));
    let preferred = preferred_base.clamp(1024, 65000);

    // 1. Probar la base preferida
    if check_port_block(preferred, count, protocol) {
        return Ok(preferred);
    }

    // 2. Probar desplazamientos seguros de 64 en 64 puertos (para evitar solapes con bloques anteriores)
    let step = 64u16;
    let mut candidate = preferred.saturating_add(step);
    while (candidate as u32) <= max_base {
        if check_port_block(candidate, count, protocol) {
            return Ok(candidate);
        }
        if candidate > 65535 - step {
            break;
        }
        candidate = candidate.saturating_add(step);
    }

    // 3. Si no se encontró hacia arriba, buscar desde 1024 hacia arriba
    let mut candidate = 1024u16;
    while (candidate as u32) < (preferred as u32) && (candidate as u32) <= max_base {
        if check_port_block(candidate, count, protocol) {
            return Ok(candidate);
        }
        candidate = candidate.saturating_add(step);
    }

    Err(format!(
        "No se encontró un bloque de {} puertos disponibles en el rango 1024..65535",
        count
    ))
}

/// Asigna y valida los rangos de puertos para un BenchmarkPlan dado.
/// En simultáneo («Ambas a la vez»):
///   - Bloque de 64 puertos: forward = base..base+31, reverse = base+32..base+63.
///   - Garantiza sin solape entre ambas direcciones (Historias.md §17).
///
/// En secuencial:
///   - forward = base..base+streams-1, reverse = base..base+streams-1.
pub fn allocate_ports_for_plan(plan: &BenchmarkPlan) -> Result<PortAllocation, String> {
    plan.validate()?;

    if plan.is_simultaneous() {
        let count = MAX_PORT_BLOCK_SIZE; // 64
        let base_port = plan.port;
        if (base_port as u32) + count - 1 > 65535 {
            return Err(format!(
                "El bloque de puertos simultáneos ({}+64) supera el puerto 65535",
                base_port
            ));
        }

        let alloc = PortAllocation {
            base_port,
            count,
            forward_range: (base_port, base_port + 31),
            reverse_range: (base_port + 32, base_port + 63),
            is_simultaneous: true,
        };

        if alloc.has_overlap() {
            return Err("Error interno: solape detectado en asignación simultánea".to_string());
        }

        Ok(alloc)
    } else {
        let count = plan.streams;
        let base_port = plan.port;
        let end_port = base_port + (count as u16).saturating_sub(1);

        Ok(PortAllocation {
            base_port,
            count,
            forward_range: (base_port, end_port),
            reverse_range: (base_port, end_port),
            is_simultaneous: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BenchmarkDirection;

    #[test]
    fn test_port_availability() {
        let port = find_available_port(18400).expect("Buscar puerto libre");
        assert!(port >= 1024);
        assert!(is_port_available(port));
    }

    #[test]
    fn test_port_allocation_sequential_no_overlap() {
        let mut plan = BenchmarkPlan::new_standard_tcp(5001);
        plan.streams = 8;
        plan.direction = BenchmarkDirection::Forward;

        let alloc = allocate_ports_for_plan(&plan).expect("Asignar puertos secuenciales");
        assert_eq!(alloc.base_port, 5001);
        assert_eq!(alloc.forward_range, (5001, 5008));
        assert_eq!(alloc.reverse_range, (5001, 5008));
        assert!(!alloc.is_simultaneous);
        assert!(!alloc.has_overlap());
    }

    #[test]
    fn test_port_allocation_simultaneous_strict_non_overlap() {
        let mut plan = BenchmarkPlan::new_standard_tcp(5001);
        plan.streams = 32;
        plan.direction = BenchmarkDirection::Both; // simultáneo

        let alloc = allocate_ports_for_plan(&plan).expect("Asignar puertos simultáneos");
        assert_eq!(alloc.base_port, 5001);
        assert_eq!(alloc.count, 64);
        assert_eq!(alloc.forward_range, (5001, 5032));
        assert_eq!(alloc.reverse_range, (5033, 5064));
        assert!(alloc.is_simultaneous);
        assert!(!alloc.has_overlap());

        // Verificar que el rango forward y reverse son disjuntos
        assert!(alloc.forward_range.1 < alloc.reverse_range.0);
    }

    #[test]
    fn test_port_block_bounds_and_udp() {
        // Bloque de 64 en 65000 cabe en u16 (65000 + 63 = 65063 <= 65535)
        let block_ok = check_port_block(65000, 64, BenchmarkProtocol::Udp);
        // Debe ejecutar sin pánico
        let _ = block_ok;

        // Bloque que supera 65535 debe ser rechazado
        assert!(!check_port_block(65500, 64, BenchmarkProtocol::Tcp));
        assert!(!check_port_block(100, 1, BenchmarkProtocol::Tcp)); // < 1024
    }
}
