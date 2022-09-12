use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

pub mod gitlab_connection;
pub mod job_analysis;
pub mod package_analysis;
pub mod package_clean;
pub mod pipeline_analysis;
pub mod pipeline_clean;

pub const STORAGE_LIMIT: u64 = 2_000_000_000;
pub const REPO_LIMIT: u64 = 100_000_000;
pub const ARTIFACT_JOBS_LIMIT: u64 = 500_000_000;
pub const ARTIFACT_JOBS_NB_LIMIT: usize = 1_000;
pub const PACKAGE_REGISTRY_LIMIT: u64 = 1_000_000_000;
pub const DOCKER_REGISTRY_LIMIT: u64 = 5_000_000_000;

pub const GITLAB_403_ERROR: &str = "403 Forbidden";
pub const GITLAB_SCOPE_ERROR: &str = "insufficient_scope";

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
    pub progress: Option<Receiver<usize>>,
    pub total: Option<usize>,
}

pub trait ReportJob {
    type Diagnosis: Reportable;
    fn diagnose(self) -> ReportPending<Self::Diagnosis>;
}

pub trait RemedyJob {
    type Report: Reportable;
    fn remedy(self) -> ReportPending<Self::Report>;
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
