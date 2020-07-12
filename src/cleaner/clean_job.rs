use crate::db;

pub struct CleanJob {}

pub struct CleanJobError {
    msg: String,
}

impl CleanJob {
    pub fn new() -> Self {
        CleanJob {}
    }

    pub fn execute(&self) -> Result<(), CleanJobError> {}
}
