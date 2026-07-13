use s4mp_core::Result;
use s4mp_pipeline::{Executor, PipelineSpec, Stage};

pub fn run() -> Result<()> {
    let spec = PipelineSpec {
        name: "default".into(),
        stages: vec![
            Stage::Import,
            Stage::Parse,
            Stage::Link,
            Stage::Analyze,
        ],
    };
    Executor::run(&spec)?;
    println!("analysis pipeline completed");
    Ok(())
}
