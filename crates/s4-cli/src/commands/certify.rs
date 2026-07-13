use s4_core::Result;

/// Certification stub.
pub fn run(policy: &str) -> Result<()> {
    println!("s4 certify --policy {policy} (stub)");
    Ok(())
}
