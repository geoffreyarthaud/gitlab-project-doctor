use crate::diagnosis::gitlab_connection::Project;
use crate::diagnosis::{Diagnosis, Report, ReportStatus};
use gitlab::Gitlab;
use human_bytes::human_bytes;

pub struct GitlabStorage<'a> {
    pub gitlab: &'a Gitlab,
    pub project: &'a Project,
    pub report: Option<Report>,
}

impl Diagnosis for GitlabStorage<'_> {
    fn diagnosis(&mut self) -> &Report {
        if self.report.is_none() {
            self.report = Some(self.analysis_storage());
        }
        &self.report.as_ref().unwrap()
    }
}

impl<'a> GitlabStorage<'a> {
    pub fn new(gitlab: &'a Gitlab, project: &'a Project) -> GitlabStorage<'a> {
        GitlabStorage {
            gitlab: gitlab,
            project: project,
            report: None,
        }
    }

    fn analysis_storage(&self) -> Report {
        let msg = format!(
            "Storage size : {}",
            human_bytes(self.project.statistics.storage_size as f64)
        );
        let status = if self.project.statistics.storage_size < 1_000_000_000 {
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
