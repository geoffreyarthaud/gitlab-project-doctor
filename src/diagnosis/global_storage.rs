use crate::diagnosis::gitlab_connection::Project;
use crate::diagnosis::repo_size::RepositorySize;
use crate::diagnosis::{Diagnosis, Report, ReportStatus, STORAGE_LIMIT};
use gitlab::Gitlab;
use human_bytes::human_bytes;
use crate::diagnosis::artifact_size::ArtifactSize;
use crate::diagnosis::package_size::PackageSize;

pub struct GlobalStorage<'a> {
    pub gitlab: &'a Gitlab,
    pub project: &'a Project,
    pub report: Option<Report>,
    pub repo_size: RepositorySize<'a>,
    pub artifact_size: ArtifactSize<'a>,
    pub package_size: PackageSize<'a>,
}

impl Diagnosis for GlobalStorage<'_> {
    fn diagnosis(&mut self) -> &Report {
        if self.report.is_none() {
            self.report = Some(self.analysis_storage());
        }
        self.report.as_ref().unwrap()
    }
}

impl<'a> GlobalStorage<'a> {
    pub fn new(gitlab: &'a Gitlab, project: &'a Project) -> GlobalStorage<'a> {
        GlobalStorage {
            gitlab,
            project,
            report: None,
            repo_size: RepositorySize::new(project),
            artifact_size: ArtifactSize::new(project),
            package_size: PackageSize::new(project),
        }
    }

    fn analysis_storage(&self) -> Report {
        let msg = format!(
            "Storage size : {}",
            human_bytes(self.project.statistics.storage_size as f64)
        );
        let status = if self.project.statistics.storage_size < STORAGE_LIMIT {
            ReportStatus::OK(msg)
        } else {
            ReportStatus::WARNING(msg)
        };
        Report {
            global: status,
            details: vec![self.repo_size.analysis_storage(),
                          self.artifact_size.analysis_storage(),
                          self.package_size.analysis_storage()],
        }
    }
}
