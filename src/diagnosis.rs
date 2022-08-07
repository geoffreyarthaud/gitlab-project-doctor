pub mod gitlab_connection;
pub mod gitlab_storage;

pub enum ReportStatus {
    OK(String),
    WARNING(String),
    ERROR(String),
}

pub struct Report {
    pub global: ReportStatus,
    pub details: Vec<ReportStatus>,
}

pub trait Diagnosis {
    fn diagnosis(&mut self) -> &Report;
}
