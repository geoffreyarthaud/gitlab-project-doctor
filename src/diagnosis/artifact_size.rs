use chrono::{DateTime, Duration, Local};
use gitlab::api::{Pagination, projects, Query};
use gitlab::api::paged;
use gitlab::Gitlab;
use human_bytes::human_bytes;
use serde::Deserialize;

use crate::{Reportable, ReportJob, ReportPending};
use crate::diagnosis::{
    ARTIFACT_JOBS_DAYS_LIMIT, ReportStatus,
};
use crate::diagnosis::gitlab_connection::{GitlabRepository, Project};

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub size: u64,
}

#[derive(Debug, Deserialize)]
pub struct GitlabJob {
    pub created_at: DateTime<Local>,
    pub artifacts: Vec<Artifact>,
}

pub struct ArtifactSizeJob {
    pub gitlab: Gitlab,
    pub project: Project,
}

pub struct ArtifactReport {
    pub gitlab_jobs: Vec<GitlabJob>,
    pub report_status: Vec<ReportStatus>,
    pub bytes_savable: u64,
}

impl Reportable for ArtifactReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl ReportJob for ArtifactSizeJob {
    type Diagnosis = ArtifactReport;

    fn diagnose(self) -> ReportPending<Self::Diagnosis> {
        ReportPending::<Self::Diagnosis> {
            pending_msg: "Analysing Gitlab jobs...".to_string(),
            job: {
                std::thread::spawn(move || {
                    if !self.project.jobs_enabled {
                        return ArtifactReport {
                            report_status: vec![
                                ReportStatus::NA("No CI/CD configured on this project".to_string())],
                            gitlab_jobs: vec![],
                            bytes_savable: 0,
                        };
                    }
                    let endpoint = projects::jobs::Jobs::builder()
                        .project(self.project.id)
                        .build()
                        .unwrap();
                    let jobs: Vec<GitlabJob> = paged(endpoint, Pagination::All).query(&self.gitlab).unwrap();
                    let (report, bytes_savable) = ArtifactSizeJob::_number_jobs(&jobs);
                    ArtifactReport {
                        report_status: vec![report],
                        gitlab_jobs: jobs,
                        bytes_savable,
                    }
                })
            },
            progress: None,
        }
    }
}

impl ArtifactSizeJob {
    pub fn from(gitlab: &GitlabRepository) -> ArtifactSizeJob {
        ArtifactSizeJob {
            gitlab: gitlab.gitlab.clone(),
            project: gitlab.project.clone(),
        }
    }

    fn _number_jobs(jobs: &[GitlabJob]) -> (ReportStatus, u64) {
        let ref_date = Local::now() - Duration::days(ARTIFACT_JOBS_DAYS_LIMIT);
        let mut old_count: usize = 0;
        let mut old_size: u64 = 0;
        for job in jobs.iter() {
            let artifact_size: u64 = job.artifacts.iter().map(|a| a.size).sum();
            if job.created_at.le(&ref_date) {
                old_count += 1;
                old_size += artifact_size;
            }
        }
        (ReportStatus::NA(format!(
            "{} jobs ({}) are older than {} days",
            old_count,
            human_bytes(old_size as f64),
            ARTIFACT_JOBS_DAYS_LIMIT
        )), old_size)
    }
}
