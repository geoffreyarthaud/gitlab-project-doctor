use human_bytes::human_bytes;

use crate::diagnosis::{ARTIFACT_JOBS_LIMIT, Diagnosis, Report, ReportStatus};
use crate::diagnosis::gitlab_connection::Project;

pub struct PackageSize<'a> {
    pub project: &'a Project,
    pub report: Option<Report>,
}

impl Diagnosis for PackageSize<'_> {
    fn diagnosis(&mut self) -> &Report {
        if self.report.is_none() {
            self.report = Some(self.analysis_storage());
        }
        self.report.as_ref().unwrap()
    }
}

impl<'a> PackageSize<'a> {
    pub fn new(project: &'a Project) -> PackageSize<'a> {
        PackageSize {
            project,
            report: None,
        }
    }

    pub fn analysis_storage(&self) -> Report {
        let msg = format!(
            "Package repository size : {} ({} %)",
            human_bytes(self.project.statistics.packages_size as f64),
            100 * self.project.statistics.packages_size / self.project.statistics.storage_size
        );
        let status = if self.project.statistics.packages_size < ARTIFACT_JOBS_LIMIT {
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
