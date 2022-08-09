use std::ops::Sub;

use chrono::{DateTime, Duration, Local};
use gitlab::api::{Pagination, projects, Query};
use gitlab::api::paged;
use gitlab::Gitlab;
use human_bytes::human_bytes;
use serde::Deserialize;

use crate::diagnosis::{ARTIFACT_JOBS_DAYS_LIMIT, ARTIFACT_JOBS_LIMIT, ARTIFACT_JOBS_NB_LIMIT, Diagnosis, Report, ReportStatus, warning_if};
use crate::diagnosis::gitlab_connection::Project;

pub struct ArtifactSize<'a> {
    pub gitlab: &'a Gitlab,
    pub project: &'a Project,
    pub report: Option<Report>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub size: u64,
}

#[derive(Debug, Deserialize)]
pub struct Job {
    pub created_at: DateTime<Local>,
    pub artifacts: Vec<Artifact>,
}

impl Diagnosis for ArtifactSize<'_> {
    fn diagnosis(&mut self) -> &Report {
        if self.report.is_none() {
            self.report = Some(self.analysis_storage());
        }
        self.report.as_ref().unwrap()
    }
}

impl<'a> ArtifactSize<'a> {
    pub fn new(gitlab: &'a Gitlab, project: &'a Project) -> ArtifactSize<'a> {
        ArtifactSize {
            gitlab,
            project,
            report: None,
        }
    }

    pub fn analysis_storage(&self) -> Report {
        let msg = format!(
            "Artifact jobs size : {} ({} %)",
            human_bytes(self.project.statistics.job_artifacts_size as f64),
            100 * self.project.statistics.job_artifacts_size / self.project.statistics.storage_size
        );
        let status =
            warning_if(self.project.statistics.job_artifacts_size > ARTIFACT_JOBS_LIMIT,
                        msg);
        let jobs = self._request_jobs();
        Report {
            global: ReportStatus::NA("Artifact Jobs".to_string()),
            details: vec![status.to_report(), self._number_jobs(&jobs)],
        }
    }

    fn _request_jobs(&self) -> Vec<Job> {
        let endpoint = projects::jobs::Jobs::builder()
            .project(self.project.id)
            .build().unwrap();
        paged(endpoint, Pagination::All)
            .query(self.gitlab)
            .unwrap()
    }

    fn _number_jobs(&self, jobs: &Vec<Job>) -> Report {
        let ref_date = Local::now() - Duration::days(ARTIFACT_JOBS_DAYS_LIMIT);
        let count_old = jobs.iter()
            .filter(|j| j.created_at.le(&ref_date))
            .count();
        Report {
            global: warning_if(jobs.len() > ARTIFACT_JOBS_NB_LIMIT,
                                format!("Number of jobs : {}", jobs.len())),
            details: vec![ReportStatus::NA(format!("{} jobs ({} %) are older than {} days",
                                                   count_old,
                                                   100*count_old/jobs.len(),
                                                   ARTIFACT_JOBS_DAYS_LIMIT)).to_report()]
        }
    }
}



