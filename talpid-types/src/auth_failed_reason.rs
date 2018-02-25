/// Possible reasons why a remote might deny access
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AuthFailedReason {
    /// The server sent an empty or unknown reason
    Unknown,
    /// The account does not exist
    InvalidAccountToken,
    /// The account has run out of time
    OutOfTime,
}
// TODO: Use the too automagic From things
impl AuthFailedReason {
    /// Turns the reason string sent by the OpenVPN server into a
    /// AuthFailedReason
    pub fn parse(_input: Option<String>) -> Self {
        // TODO: When our servers send reasons why they deny access this is where
        // we wil parse that reason.
        AuthFailedReason::Unknown
    }
}

