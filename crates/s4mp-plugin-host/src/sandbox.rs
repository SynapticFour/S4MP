/// Plugin trust tier determines sandbox policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustTier {
    TrustedFirstParty,
    SignedThirdParty,
    UntrustedCommunity,
}

/// Sandbox configuration for plugin execution.
#[derive(Clone, Debug)]
pub struct Sandbox {
    pub tier: TrustTier,
    pub allow_network: bool,
}

impl Sandbox {
    pub fn for_tier(tier: TrustTier) -> Self {
        Self {
            tier,
            allow_network: matches!(tier, TrustTier::TrustedFirstParty),
        }
    }
}
