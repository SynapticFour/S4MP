#[derive(Clone, Debug)]
pub enum Stage {
    Import,
    Parse,
    Link,
    Analyze,
    Reason,
    Verify,
}
