use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub ip_address: String,
    pub is_loopback: bool,
    pub speed_bps: Option<u64>,
}

/// Determina la mejor interfaz local para enrutar tráfico hacia `target_ip`
/// utilizando la tabla de rutas del sistema operativo mediante un socket UDP no conectado.
pub fn find_best_interface_for_target(target_ip: IpAddr) -> Result<NetworkInterfaceInfo, String> {
    let dummy_target = match target_ip {
        IpAddr::V4(v4) => SocketAddr::new(IpAddr::V4(v4), 5001),
        IpAddr::V6(v6) => SocketAddr::new(IpAddr::V6(v6), 5001),
    };

    let bind_addr: SocketAddr = match target_ip {
        IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
        IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0),
    };

    let socket = UdpSocket::bind(bind_addr).map_err(|e| format!("Fallo al crear socket local: {}", e))?;
    socket
        .connect(dummy_target)
        .map_err(|e| format!("Fallo al consultar ruta hacia {}: {}", target_ip, e))?;

    let local_addr = socket
        .local_addr()
        .map_err(|e| format!("Fallo al obtener dirección local del socket: {}", e))?;

    let ip = local_addr.ip();
    let is_loopback = ip.is_loopback();
    let name = if is_loopback {
        "Loopback Pseudo-Interface".to_string()
    } else {
        format!("Interfaz ({})", ip)
    };

    Ok(NetworkInterfaceInfo {
        name,
        ip_address: ip.to_string(),
        is_loopback,
        speed_bps: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_best_interface_for_localhost() {
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let iface = find_best_interface_for_target(localhost).expect("Debe resolver ruta local");
        assert!(iface.is_loopback || iface.ip_address == "127.0.0.1");
    }
}
