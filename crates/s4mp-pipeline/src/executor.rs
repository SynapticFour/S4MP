use crate::PipelineSpec;
use s4mp_core::Result;

pub struct Executor;

impl Executor {
    pub fn run(spec: &PipelineSpec) -> Result<()> {
        for stage in &spec.stages {
            let _ = stage;
        }
        Ok(())
    }
}
