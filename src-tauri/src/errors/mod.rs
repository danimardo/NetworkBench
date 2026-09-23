use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "fatal")]
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "NB-CONN-001")]
    ConnCannotReach,
    #[serde(rename = "NB-CONN-002")]
    ConnDnsUnresolvable,
    #[serde(rename = "NB-CONN-003")]
    ConnRejected,
    #[serde(rename = "NB-CONN-004")]
    ConnNotNetworkBench,
    #[serde(rename = "NB-CONN-005")]
    ConnInterrupted,
    #[serde(rename = "NB-CONN-006")]
    ConnTlsFailure,

    #[serde(rename = "NB-VERSION-001")]
    VersionIncompatible,

    #[serde(rename = "NB-PEER-001")]
    PeerRejected,
    #[serde(rename = "NB-PEER-002")]
    PeerPairingExpired,
    #[serde(rename = "NB-PEER-003")]
    PeerBusy,
    #[serde(rename = "NB-PEER-004")]
    PeerCancelled,
    #[serde(rename = "NB-PEER-005")]
    PeerPairingMismatch,
    #[serde(rename = "NB-PEER-006")]
    PeerIdentityChanged,

    #[serde(rename = "NB-PARAM-001")]
    ParamRejected,

    #[serde(rename = "NB-PORT-001")]
    PortControlInUse,
    #[serde(rename = "NB-PORT-002")]
    PortDataRangeInUse,

    #[serde(rename = "NB-FW-001")]
    FirewallBlockedControl,
    #[serde(rename = "NB-FW-002")]
    FirewallBlockedNtttcp,
    #[serde(rename = "NB-FW-003")]
    FirewallPolicyManaged,
    #[serde(rename = "NB-FW-004")]
    FirewallUacRejected,
    #[serde(rename = "NB-FW-005")]
    FirewallExternalBlocked,

    #[serde(rename = "NB-ENGINE-001")]
    EngineNotFound,
    #[serde(rename = "NB-ENGINE-002")]
    EngineHashMismatch,
    #[serde(rename = "NB-ENGINE-003")]
    EngineReceiverNotListening,
    #[serde(rename = "NB-ENGINE-004")]
    EngineTimeout,
    #[serde(rename = "NB-ENGINE-005")]
    EngineTerminatedError,
    #[serde(rename = "NB-ENGINE-006")]
    EngineXmlInvalid,

    #[serde(rename = "NB-RESULT-001")]
    ResultIncomplete,
    #[serde(rename = "NB-RESULT-002")]
    ResultInconsistent,

    #[serde(rename = "NB-NIC-001")]
    NicDisconnected,
    #[serde(rename = "NB-NIC-002")]
    NicStateChanged,
    #[serde(rename = "NB-NIC-003")]
    NicIpFamilyMismatch,
    #[serde(rename = "NB-NIC-004")]
    NicNoneConnected,

    #[serde(rename = "NB-PERM-001")]
    PermissionDenied,

    #[serde(rename = "NB-DISK-001")]
    DiskSpaceLow,

    #[serde(rename = "NB-DATA-001")]
    DatabaseCorrupt,

    #[serde(rename = "NB-UNEXPECTED-001")]
    UnexpectedError,

    #[serde(rename = "NB-INTERNAL-001")]
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorAction {
    #[serde(rename = "retry")]
    Retry,
    #[serde(rename = "repeat_test")]
    RepeatTest,
    #[serde(rename = "show_details")]
    ShowDetails,
    #[serde(rename = "change_address")]
    ChangeAddress,
    #[serde(rename = "change_port")]
    ChangePort,
    #[serde(rename = "change_ports")]
    ChangePorts,
    #[serde(rename = "retry_pairing")]
    RetryPairing,
    #[serde(rename = "retry_later")]
    RetryLater,
    #[serde(rename = "reverify_identity")]
    ReverifyIdentity,
    #[serde(rename = "check_install")]
    CheckInstall,
    #[serde(rename = "reinstall")]
    Reinstall,
    #[serde(rename = "open_firewall_settings")]
    OpenFirewallSettings,
    #[serde(rename = "configure_firewall")]
    ConfigureFirewall,
    #[serde(rename = "show_firewall_instructions")]
    ShowFirewallInstructions,
    #[serde(rename = "retry_as_admin")]
    RetryAsAdmin,
    #[serde(rename = "choose_interface")]
    ChooseInterface,
    #[serde(rename = "open_network_settings")]
    OpenNetworkSettings,
    #[serde(rename = "free_space")]
    FreeSpace,
    #[serde(rename = "update_app")]
    UpdateApp,
    #[serde(rename = "copy_diagnostic")]
    CopyDiagnostic,
    #[serde(rename = "copy_diagnostics")]
    CopyDiagnostics,
    #[serde(rename = "check_connection")]
    CheckConnection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppIssue {
    pub path: String,
    pub code: String,
    pub message_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    pub severity: ErrorSeverity,
    pub message_key: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<AppIssue>,
    #[serde(default)]
    pub actions: Vec<ErrorAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_id: Option<String>,
}

impl AppError {
    pub fn new(code: ErrorCode, severity: ErrorSeverity, message_key: impl Into<String>) -> Self {
        Self {
            code,
            severity,
            message_key: message_key.into(),
            issues: Vec::new(),
            actions: Vec::new(),
            diagnostic_id: None,
        }
    }

    pub fn with_action(mut self, action: ErrorAction) -> Self {
        if !self.actions.contains(&action) {
            self.actions.push(action);
        }
        self
    }

    pub fn with_actions(mut self, actions: impl IntoIterator<Item = ErrorAction>) -> Self {
        for action in actions {
            self = self.with_action(action);
        }
        self
    }

    pub fn with_diagnostic_id(mut self, id: impl Into<String>) -> Self {
        self.diagnostic_id = Some(id.into());
        self
    }

    pub fn with_issue(mut self, issue: AppIssue) -> Self {
        self.issues.push(issue);
        self
    }

    pub fn from_code(code: ErrorCode) -> Self {
        match code {
            ErrorCode::ConnCannotReach => Self::new(code, ErrorSeverity::Error, "errors.NB-CONN-001")
                .with_actions([ErrorAction::Retry, ErrorAction::ChangeAddress, ErrorAction::ShowDetails]),
            ErrorCode::ConnDnsUnresolvable => Self::new(code, ErrorSeverity::Error, "errors.NB-CONN-002")
                .with_actions([ErrorAction::ChangeAddress, ErrorAction::ShowDetails]),
            ErrorCode::ConnRejected => Self::new(code, ErrorSeverity::Error, "errors.NB-CONN-003")
                .with_actions([ErrorAction::Retry, ErrorAction::ShowDetails]),
            ErrorCode::ConnNotNetworkBench => Self::new(code, ErrorSeverity::Error, "errors.NB-CONN-004")
                .with_actions([ErrorAction::ChangePort, ErrorAction::ShowDetails]),
            ErrorCode::ConnInterrupted => Self::new(code, ErrorSeverity::Error, "errors.NB-CONN-005")
                .with_actions([ErrorAction::RepeatTest, ErrorAction::CheckConnection]),
            ErrorCode::ConnTlsFailure => Self::new(code, ErrorSeverity::Error, "errors.NB-CONN-006")
                .with_actions([ErrorAction::ShowDetails]),

            ErrorCode::VersionIncompatible => Self::new(code, ErrorSeverity::Error, "errors.NB-VERSION-001")
                .with_actions([ErrorAction::UpdateApp]),

            ErrorCode::PeerRejected => Self::new(code, ErrorSeverity::Info, "errors.NB-PEER-001")
                .with_actions([ErrorAction::Retry]),
            ErrorCode::PeerPairingExpired => Self::new(code, ErrorSeverity::Warning, "errors.NB-PEER-002")
                .with_actions([ErrorAction::Retry]),
            ErrorCode::PeerBusy => Self::new(code, ErrorSeverity::Warning, "errors.NB-PEER-003")
                .with_actions([ErrorAction::Retry]),
            ErrorCode::PeerCancelled => Self::new(code, ErrorSeverity::Info, "errors.NB-PEER-004")
                .with_actions([ErrorAction::RepeatTest]),
            ErrorCode::PeerPairingMismatch => Self::new(code, ErrorSeverity::Error, "errors.NB-PEER-005")
                .with_actions([ErrorAction::Retry, ErrorAction::ShowDetails]),
            ErrorCode::PeerIdentityChanged => Self::new(code, ErrorSeverity::Error, "errors.NB-PEER-006")
                .with_actions([ErrorAction::ReverifyIdentity, ErrorAction::ShowDetails]),

            ErrorCode::ParamRejected => Self::new(code, ErrorSeverity::Error, "errors.NB-PARAM-001")
                .with_actions([ErrorAction::ShowDetails]),

            ErrorCode::PortControlInUse => Self::new(code, ErrorSeverity::Error, "errors.NB-PORT-001")
                .with_actions([ErrorAction::ChangePort]),
            ErrorCode::PortDataRangeInUse => Self::new(code, ErrorSeverity::Error, "errors.NB-PORT-002")
                .with_actions([ErrorAction::ChangePort, ErrorAction::ShowDetails]),

            ErrorCode::FirewallBlockedControl => Self::new(code, ErrorSeverity::Error, "errors.NB-FW-001")
                .with_actions([ErrorAction::ShowFirewallInstructions, ErrorAction::Retry]),
            ErrorCode::FirewallBlockedNtttcp => Self::new(code, ErrorSeverity::Error, "errors.NB-FW-002")
                .with_actions([ErrorAction::ConfigureFirewall, ErrorAction::ShowFirewallInstructions]),
            ErrorCode::FirewallPolicyManaged => Self::new(code, ErrorSeverity::Error, "errors.NB-FW-003")
                .with_actions([ErrorAction::ShowFirewallInstructions, ErrorAction::CopyDiagnostics]),
            ErrorCode::FirewallUacRejected => Self::new(code, ErrorSeverity::Warning, "errors.NB-FW-004")
                .with_actions([ErrorAction::RetryAsAdmin, ErrorAction::ShowFirewallInstructions]),
            ErrorCode::FirewallExternalBlocked => Self::new(code, ErrorSeverity::Error, "errors.NB-FW-005")
                .with_actions([ErrorAction::ShowFirewallInstructions, ErrorAction::ShowDetails]),

            ErrorCode::EngineNotFound => Self::new(code, ErrorSeverity::Fatal, "errors.NB-ENGINE-001")
                .with_actions([ErrorAction::ShowDetails, ErrorAction::Reinstall]),
            ErrorCode::EngineHashMismatch => Self::new(code, ErrorSeverity::Fatal, "errors.NB-ENGINE-002")
                .with_actions([ErrorAction::ShowDetails]),
            ErrorCode::EngineReceiverNotListening => Self::new(code, ErrorSeverity::Error, "errors.NB-ENGINE-003")
                .with_actions([ErrorAction::RepeatTest, ErrorAction::ChangePort, ErrorAction::ShowDetails]),
            ErrorCode::EngineTimeout => Self::new(code, ErrorSeverity::Error, "errors.NB-ENGINE-004")
                .with_actions([ErrorAction::RepeatTest, ErrorAction::ShowDetails]),
            ErrorCode::EngineTerminatedError => Self::new(code, ErrorSeverity::Error, "errors.NB-ENGINE-005")
                .with_actions([ErrorAction::RepeatTest, ErrorAction::ShowDetails]),
            ErrorCode::EngineXmlInvalid => Self::new(code, ErrorSeverity::Error, "errors.NB-ENGINE-006")
                .with_actions([ErrorAction::RepeatTest, ErrorAction::ShowDetails]),

            ErrorCode::ResultIncomplete => Self::new(code, ErrorSeverity::Warning, "errors.NB-RESULT-001")
                .with_actions([ErrorAction::RepeatTest]),
            ErrorCode::ResultInconsistent => Self::new(code, ErrorSeverity::Warning, "errors.NB-RESULT-002")
                .with_actions([ErrorAction::RepeatTest, ErrorAction::ShowDetails]),

            ErrorCode::NicDisconnected => Self::new(code, ErrorSeverity::Error, "errors.NB-NIC-001")
                .with_actions([ErrorAction::ChooseInterface, ErrorAction::OpenNetworkSettings]),
            ErrorCode::NicStateChanged => Self::new(code, ErrorSeverity::Error, "errors.NB-NIC-002")
                .with_actions([ErrorAction::RepeatTest, ErrorAction::ChooseInterface]),
            ErrorCode::NicIpFamilyMismatch => Self::new(code, ErrorSeverity::Error, "errors.NB-NIC-003")
                .with_actions([ErrorAction::ChooseInterface, ErrorAction::ShowDetails]),
            ErrorCode::NicNoneConnected => Self::new(code, ErrorSeverity::Error, "errors.NB-NIC-004")
                .with_actions([ErrorAction::OpenNetworkSettings]),

            ErrorCode::PermissionDenied => Self::new(code, ErrorSeverity::Error, "errors.NB-PERM-001")
                .with_actions([ErrorAction::RetryAsAdmin, ErrorAction::ShowDetails]),

            ErrorCode::DiskSpaceLow => Self::new(code, ErrorSeverity::Warning, "errors.NB-DISK-001")
                .with_actions([ErrorAction::FreeSpace]),

            ErrorCode::DatabaseCorrupt => Self::new(code, ErrorSeverity::Fatal, "errors.NB-DATA-001")
                .with_actions([ErrorAction::ShowDetails]),

            ErrorCode::UnexpectedError => Self::new(code, ErrorSeverity::Error, "errors.NB-UNEXPECTED-001")
                .with_actions([ErrorAction::CopyDiagnostics, ErrorAction::Retry]),

            ErrorCode::InternalError => Self::new(code, ErrorSeverity::Error, "errors.NB-INTERNAL-001")
                .with_actions([ErrorAction::CopyDiagnostics, ErrorAction::Retry]),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message_key)
    }
}

impl std::error::Error for AppError {}
