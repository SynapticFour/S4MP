/// Policy constraints for a reasoning request.
#[derive(Clone, Debug, Default)]
pub struct ReasonPolicy {
    /// Maximum output tokens, if limited.
    pub max_tokens: Option<u32>,
    /// Expected output schema identifier.
    pub output_schema: Option<String>,
    /// Whether network access is permitted (sandbox policy).
    pub allow_network: bool,
}
