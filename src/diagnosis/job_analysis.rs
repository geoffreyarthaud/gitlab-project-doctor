use chrono::{DateTime, Duration, Local};
use gitlab::api::paged;
use gitlab::api::{projects, Pagination, Query};
use gitlab::Gitlab;
use human_bytes::human_bytes;
use serde::Deserialize;

use crate::diagnosis::gitlab_connection::{GitlabRepository, Project};
use crate::diagnosis::ReportStatus;
use crate::{ReportJob, ReportPending, Reportable};

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub size: u64,
}

#[derive(Debug, Deserialize)]
pub struct GitlabJob {
    pub created_at: DateTime<Local>,
    pub artifacts: Vec<Artifact>,
}

pub struct JobAnalysisJob {
    pub gitlab: Gitlab,
    pub project: Project,
    pub days: usize,
}

pub struct JobAnalysisReport {
    pub gitlab_jobs: Vec<GitlabJob>,
    pub report_status: Vec<ReportStatus>,
    pub savable_bytes: u64,
}

impl Reportable for JobAnalysisReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl ReportJob for JobAnalysisJob {
    type Diagnosis = JobAnalysisReport;

    fn diagnose(self) -> ReportPending<Self::Diagnosis> {
        ReportPending::<Self::Diagnosis> {
            pending_msg: "Analysing Gitlab jobs...".to_string(),
            job: {
                std::thread::spawn(move || {
                    if !self.project.jobs_enabled {
                        return JobAnalysisReport {
                            report_status: vec![ReportStatus::NA(
                                "No CI/CD configured on this project".to_string(),
                            )],
                            gitlab_jobs: vec![],
                            savable_bytes: 0,
                        };
                    }
                    let endpoint = projects::jobs::Jobs::builder()
                        .project(self.project.id)
                        .build()
                        .unwrap();
                    let jobs: Vec<GitlabJob> = paged(endpoint, Pagination::All)
                        .query(&self.gitlab)
                        .unwrap();
                    let (report, bytes_savable) = self._number_jobs(&jobs);
                    JobAnalysisReport {
                        report_status: vec![report],
                        gitlab_jobs: jobs,
                        savable_bytes: bytes_savable,
                    }
                })
            },
            progress: None,
            total: None,
        }
    }
}

impl JobAnalysisJob {
    pub fn from(gitlab: &GitlabRepository, days: usize) -> JobAnalysisJob {
        JobAnalysisJob {
            gitlab: gitlab.gitlab.clone(),
            project: gitlab.project.clone(),
            days,
        }
    }

    fn _number_jobs(&self, jobs: &[GitlabJob]) -> (ReportStatus, u64) {
        let ref_date = Local::now() - Duration::days(self.days as i64);
        let mut old_count: usize = 0;
        let mut old_size: u64 = 0;
        for job in jobs.iter() {
            let artifact_size: u64 = job.artifacts.iter().map(|a| a.size).sum();
            if job.created_at.le(&ref_date) {
                old_count += 1;
                old_size += artifact_size;
            }
        }
        (
            ReportStatus::NA(format!(
                "{} jobs ({}) are older than {} days",
                old_count,
                human_bytes(old_size as f64),
                self.days
            )),
            old_size,
        )
    }
}
