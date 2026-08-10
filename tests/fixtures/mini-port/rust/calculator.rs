//! Tiny fixture for S4MP e2e — not production code.

/// Mirrors the Java `Calculator` surface for heuristic name matching.
pub struct Calculator;

impl Calculator {
    /// Add two integers.
    pub fn add(a: i32, b: i32) -> i32 {
        helper(a) + scale(b)
    }

    /// Multiply two integers.
    pub fn multiply(a: i32, b: i32) -> i32 {
        a * b
    }

    /// Extra in Rust (no Java counterpart) for ExtraInTarget coverage.
    pub fn subtract(a: i32, b: i32) -> i32 {
        a - b
    }
}

fn helper(x: i32) -> i32 {
    x
}

fn scale(x: i32) -> i32 {
    x * 2
}
