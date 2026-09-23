use crate::control::ports::check_port_range;
use crate::errors::{AppError, ErrorCode};
use crate::firewall::RuleStatus;
use crate::netinfo::find_best_interface_for_target;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::net::IpAddr;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PreflightCheckType {
    Engine,
    NetworkInterface,
    Ports,
    Firewall,
    DiskSpace,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PreflightStatus {
    Pending,
    Running,
    Passed,
    Warning,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightCheck {
    pub check_type: PreflightCheckType,
    pub status: PreflightStatus,
    pub title_key: String,
    pub description_key: String,
    pub error: Option<AppError>,
}

impl PreflightCheck {
    pub fn passed(check_type: PreflightCheckType, title_key: &str, desc_key: &str) -> Self {
        Self {
            check_type,
            status: PreflightStatus::Passed,
            title_key: title_key.to_string(),
            description_key: desc_key.to_string(),
            error: None,
        }
    }

    pub fn warning(
        check_type: PreflightCheckType,
        title_key: &str,
        desc_key: &str,
        err: AppError,
    ) -> Self {
        Self {
            check_type,
            status: PreflightStatus::Warning,
            title_key: title_key.to_string(),
            description_key: desc_key.to_string(),
            error: Some(err),
        }
    }

    pub fn failed(
        check_type: PreflightCheckType,
        title_key: &str,
        desc_key: &str,
        err: AppError,
    ) -> Self {
        Self {
            check_type,
            status: PreflightStatus::Failed,
            title_key: title_key.to_string(),
            description_key: desc_key.to_string(),
            error: Some(err),
        }
    }
}

pub struct PreflightEvaluator;

impl PreflightEvaluator {
    /// 1. Comprobación del motor NTTTCP (FR-019, §11, §28)
    pub fn check_engine(engine_path: &Path, expected_hash: &str) -> PreflightCheck {
        if !engine_path.exists() {
            return PreflightCheck::failed(
                PreflightCheckType::Engine,
                "preflight.engine",
                "errors.NB_ENGINE_001.desc",
                AppError::from_code(ErrorCode::EngineNotFound),
            );
        }

        let bytes = match fs::read(engine_path) {
            Ok(b) => b,
            Err(_) => {
                return PreflightCheck::failed(
                    PreflightCheckType::Engine,
                    "preflight.engine",
                    "errors.NB_ENGINE_002.desc",
                    AppError::from_code(ErrorCode::EngineHashMismatch),
                );
            }
        };

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash_bytes = hasher.finalize();
        let actual_hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        if !expected_hash.is_empty() && actual_hash.to_lowercase() != expected_hash.to_lowercase() {
            return PreflightCheck::failed(
                PreflightCheckType::Engine,
                "preflight.engine",
                "errors.NB_ENGINE_002.desc",
                AppError::from_code(ErrorCode::EngineHashMismatch),
            );
        }

        PreflightCheck::passed(
            PreflightCheckType::Engine,
            "preflight.engine",
            "preflight.status_ok",
        )
    }

    /// 2. Comprobación del adaptador de red y ruta (FR-019, §10.2, §15, §28)
    pub fn check_nic(target_ip: IpAddr) -> PreflightCheck {
        match find_best_interface_for_target(target_ip) {
            Ok(_) => PreflightCheck::passed(
                PreflightCheckType::NetworkInterface,
                "preflight.nic",
                "preflight.status_ok",
            ),
            Err(e) => {
                let code = if e.contains("familia") || e.contains("family") {
                    ErrorCode::NicIpFamilyMismatch
                } else {
                    ErrorCode::NicNoneConnected
                };
                PreflightCheck::failed(
                    PreflightCheckType::NetworkInterface,
                    "preflight.nic",
                    "errors.NB_NIC_004.desc",
                    AppError::from_code(code).with_diagnostic_id(e),
                )
            }
        }
    }

    /// 3. Comprobación de disponibilidad de puertos (FR-020, §10.3, §28)
    pub fn check_ports(control_port: u16, data_base_port: u16, streams: u32) -> PreflightCheck {
        if !crate::control::ports::is_port_available(control_port) {
            return PreflightCheck::failed(
                PreflightCheckType::Ports,
                "preflight.ports",
                "errors.NB_PORT_001.desc",
                AppError::from_code(ErrorCode::PortControlInUse),
            );
        }

        if !check_port_range(data_base_port, streams) {
            return PreflightCheck::failed(
                PreflightCheckType::Ports,
                "preflight.ports",
                "errors.NB_PORT_002.desc",
                AppError::from_code(ErrorCode::PortDataRangeInUse),
            );
        }

        PreflightCheck::passed(
            PreflightCheckType::Ports,
            "preflight.ports",
            "preflight.status_ok",
        )
    }

    /// 4. Comprobación de espacio libre en disco (%APPDATA% >= 200 MB, §19.1, §28)
    pub fn check_disk_space(dir: &Path, min_mb: u64) -> PreflightCheck {
        let free_bytes = Self::get_free_disk_space(dir).unwrap_or(u64::MAX);
        let min_bytes = min_mb * 1024 * 1024;

        if free_bytes < min_bytes {
            PreflightCheck::warning(
                PreflightCheckType::DiskSpace,
                "preflight.disk",
                "errors.NB_DISK_001.desc",
                AppError::from_code(ErrorCode::DiskSpaceLow),
            )
        } else {
            PreflightCheck::passed(
                PreflightCheckType::DiskSpace,
                "preflight.disk",
                "preflight.status_ok",
            )
        }
    }

    /// 5. Comprobación de versiones del protocolo (FR-019, §28)
    pub fn check_version(local_protocol: u32, peer_protocol: u32) -> PreflightCheck {
        if local_protocol != peer_protocol {
            PreflightCheck::failed(
                PreflightCheckType::Version,
                "preflight.version",
                "errors.NB_VERSION_001.desc",
                AppError::from_code(ErrorCode::VersionIncompatible),
            )
        } else {
            PreflightCheck::passed(
                PreflightCheckType::Version,
                "preflight.version",
                "preflight.status_ok",
            )
        }
    }

    /// 6. Comprobación de firewall según canal de control y sondeo TCP (§14.4, §28)
    pub fn check_firewall(
        is_control_ok: bool,
        test_probe_ok: bool,
        rule_status: RuleStatus,
    ) -> PreflightCheck {
        if !is_control_ok {
            return PreflightCheck::failed(
                PreflightCheckType::Firewall,
                "preflight.firewall",
                "errors.NB_FW_001.desc",
                AppError::from_code(ErrorCode::FirewallBlockedControl),
            );
        }

        if !test_probe_ok {
            if rule_status == RuleStatus::Missing || rule_status == RuleStatus::Disabled {
                return PreflightCheck::failed(
                    PreflightCheckType::Firewall,
                    "preflight.firewall",
                    "errors.NB_FW_002.desc",
                    AppError::from_code(ErrorCode::FirewallBlockedNtttcp),
                );
            } else {
                return PreflightCheck::failed(
                    PreflightCheckType::Firewall,
                    "preflight.firewall",
                    "errors.NB_FW_005.desc",
                    AppError::from_code(ErrorCode::FirewallExternalBlocked),
                );
            }
        }

        PreflightCheck::passed(
            PreflightCheckType::Firewall,
            "preflight.firewall",
            "preflight.status_ok",
        )
    }

    #[cfg(target_os = "windows")]
    fn get_free_disk_space(path: &Path) -> Option<u64> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

        let path_str = path.to_str()?;
        let wide: Vec<u16> = OsStr::new(path_str)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes_available: u64 = 0;
        let mut total_number_of_bytes: u64 = 0;
        let mut total_number_of_free_bytes: u64 = 0;

        unsafe {
            let res = GetDiskFreeSpaceExW(
                windows::core::PCWSTR(wide.as_ptr()),
                Some(&mut free_bytes_available),
                Some(&mut total_number_of_bytes),
                Some(&mut total_number_of_free_bytes),
            );

            if res.is_ok() {
                Some(free_bytes_available)
            } else {
                None
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_free_disk_space(_path: &Path) -> Option<u64> {
        Some(10 * 1024 * 1024 * 1024) // 10 GB en pruebas no Windows
    }
}
