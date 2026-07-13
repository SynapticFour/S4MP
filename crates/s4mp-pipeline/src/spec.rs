use crate::Stage;

#[derive(Clone, Debug, Default)]
pub struct PipelineSpec {
    pub name: String,
    pub stages: Vec<Stage>,
}
