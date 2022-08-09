use human_bytes::human_bytes;

use crate::diagnosis::{Diagnosis, REPO_LIMIT, Report, ReportStatus};
use crate::diagnosis::gitlab_connection::Project;

pub struct RepositorySize<'a> {
    pub project: &'a Project,
    pub report: Option<Report>,
}

impl Diagnosis for RepositorySize<'_> {
    fn diagnosis(&mut self) -> &Report {
        if self.report.is_none() {
            self.report = Some(self.analysis_storage());
        }
        self.report.as_ref().unwrap()
    }
}

impl<'a> RepositorySize<'a> {
    pub fn new(project: &'a Project) -> RepositorySize<'a> {
        RepositorySize {
            project,
            report: None,
        }
    }

    pub fn analysis_storage(&self) -> Report {
        let msg = format!(
            "Git repository size : {} ({} %)",
            human_bytes(self.project.statistics.repository_size as f64),
            100 * self.project.statistics.repository_size / self.project.statistics.storage_size
        );
        let status = if self.project.statistics.repository_size < REPO_LIMIT {
            ReportStatus::OK(msg)
        } else {
            ReportStatus::WARNING(msg)
        };
        Report {
            global: status,
            details: vec![],
        }
    }
}
