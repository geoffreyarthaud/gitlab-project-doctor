pub mod artifact_size;
pub mod gitlab_connection;
pub mod global_storage;
pub mod package_size;
pub mod repo_size;

pub const STORAGE_LIMIT: u64 = 2_000_000_000;
pub const REPO_LIMIT: u64 = 100_000_000;
pub const ARTIFACT_JOBS_LIMIT: u64 = 500_000_000;
pub const ARTIFACT_JOBS_NB_LIMIT: usize = 1_000;
pub const ARTIFACT_JOBS_DAYS_LIMIT: i64 = 30;
pub const PACKAGE_REGISTRY_LIMIT: u64 = 1_000_000_000;
pub const DOCKER_REGISTRY_LIMIT: u64 = 5_000_000_000;

pub enum ReportStatus {
    OK(String),
    WARNING(String),
    ERROR(String),
    NA(String),
    PENDING(String),
}
impl ReportStatus {
    fn to_report(self) -> Report {
        Report {
            global: self,
            details: vec![],
        }
    }
}

pub struct Report {
    pub global: ReportStatus,
    pub details: Vec<Report>,
}

pub trait Diagnosis {
    fn diagnosis(&mut self) -> &Report;
}

pub fn warning_if(condition: bool, message: String) -> ReportStatus {
    if condition {
        ReportStatus::WARNING(message)
    } else {
        ReportStatus::OK(message)
    }
}
