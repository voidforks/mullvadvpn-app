use talpid_types::AuthFailedReason;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelExitCause {
    None,
    AuthFailed(AuthFailedReason),
}

