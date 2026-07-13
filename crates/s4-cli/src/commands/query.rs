use s4_core::Result;

/// Query stub.
pub fn run(expr: &str) -> Result<()> {
    println!("s4 query --expr {expr} (stub)");
    Ok(())
}
