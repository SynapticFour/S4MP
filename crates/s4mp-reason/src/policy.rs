#[derive(Clone, Debug, Default)]
pub struct ReasonPolicy {
    pub max_tokens: Option<u32>,
    pub output_schema: Option<String>,
}
