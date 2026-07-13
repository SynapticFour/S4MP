use crate::{Job, JobId, JobStatus};

pub struct Scheduler {
    next_id: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn enqueue(&mut self, name: impl Into<String>) -> Job {
        let id = JobId(self.next_id);
        self.next_id += 1;
        Job {
            id,
            name: name.into(),
            status: JobStatus::Pending,
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
