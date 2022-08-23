use chrono::{DateTime, Duration, Local};
use gitlab::api::{Pagination, projects, Query};
use gitlab::api::paged;
use gitlab::Gitlab;
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
                        };
                    }
                    let endpoint = projects::jobs::Jobs::builder()
                        .project(self.project.id)
                        .build()
                        .unwrap();
                    let jobs: Vec<GitlabJob> = paged(endpoint, Pagination::All).query(&self.gitlab).unwrap();
                    ArtifactReport {
                        report_status: ArtifactSizeJob::_number_jobs(&jobs),
                        gitlab_jobs: jobs,
                    }
                })
            },
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

    fn _number_jobs(jobs: &Vec<GitlabJob>) -> Vec<ReportStatus> {
        let ref_date = Local::now() - Duration::days(ARTIFACT_JOBS_DAYS_LIMIT);
        let count_old = jobs.iter().filter(|j| j.created_at.le(&ref_date)).count();
        vec![ReportStatus::NA(format!(
            "{} jobs ({} %) are older than {} days",
            count_old,
            100 * count_old / jobs.len(),
            ARTIFACT_JOBS_DAYS_LIMIT
        ))]
    }
}
