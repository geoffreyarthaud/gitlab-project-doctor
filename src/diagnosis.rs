use std::thread::JoinHandle;

pub mod artifact_size;
pub mod gitlab_connection;

pub const STORAGE_LIMIT: u64 = 2_000_000_000;
pub const REPO_LIMIT: u64 = 100_000_000;
pub const ARTIFACT_JOBS_LIMIT: u64 = 500_000_000;
pub const ARTIFACT_JOBS_NB_LIMIT: usize = 1_000;
pub const ARTIFACT_JOBS_DAYS_LIMIT: i64 = 30;
pub const PACKAGE_REGISTRY_LIMIT: u64 = 1_000_000_000;
pub const DOCKER_REGISTRY_LIMIT: u64 = 5_000_000_000;

#[derive(Clone)]
pub enum ReportStatus {
    OK(String),
    WARNING(String),
    ERROR(String),
    NA(String),
}

pub struct ReportPending<T> {
    pub pending_msg: String,
    pub job: JoinHandle<T>,
}

pub trait ReportJob {
    type Diagnosis: Reportable;
    fn diagnose(self) -> ReportPending<Self::Diagnosis>;
}

pub trait Reportable {
    fn report(&self) -> Vec<ReportStatus>;
}

pub fn warning_if(condition: bool, message: String) -> ReportStatus {
    if condition {
        ReportStatus::WARNING(message)
    } else {
        ReportStatus::OK(message)
    }
}
