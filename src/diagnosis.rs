pub mod gitlab_connection;
pub mod global_storage;
pub mod repo_size;
pub mod artifact_size;
pub mod package_size;

pub const STORAGE_LIMIT : u64 = 2_000_000_000;
pub const REPO_LIMIT : u64 = 100_000_000;
pub const ARTIFACT_JOBS_LIMIT: u64 = 500_000_000;
pub const PACKAGE_REGISTRY_LIMIT: u64 = 1_000_000_000;
pub const DOCKER_REGISTRY_LIMIT: u64 = 5_000_000_000;

pub enum ReportStatus {
    OK(String),
    WARNING(String),
    ERROR(String),
}

pub struct Report {
    pub global: ReportStatus,
    pub details: Vec<Report>,
}

pub trait Diagnosis {
    fn diagnosis(&mut self) -> &Report;
}
