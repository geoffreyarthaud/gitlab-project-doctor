use chrono::{DateTime, Duration, Local};
use gitlab::api::{Pagination, Query};
use gitlab::Gitlab;
use serde::Deserialize;

use crate::{Reportable, ReportJob, ReportPending, ReportStatus};
use crate::diagnosis::ARTIFACT_JOBS_DAYS_LIMIT;
use crate::diagnosis::gitlab_connection::{GitlabRepository, Project};

#[derive(Debug, Deserialize)]
pub struct GitlabPipeline {
    pub id: u64,
    pub created_at: DateTime<Local>,
}

pub struct PipelineAnalysisJob {
    pub gitlab: Gitlab,
    pub project: Project,
}

pub struct PipelineAnalysisReport {
    pub gitlab: Gitlab,
    pub project: Project,
    pub pipelines: Vec<GitlabPipeline>,
    pub report_status: Vec<ReportStatus>,
}

impl Reportable for PipelineAnalysisReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl PipelineAnalysisJob {
    fn to_report(self, report_status: Vec<ReportStatus>, pipelines: Vec<GitlabPipeline>)
        -> PipelineAnalysisReport {
        PipelineAnalysisReport {
            gitlab: self.gitlab,
            project: self.project,
            pipelines,
            report_status
        }
    }
}
impl ReportJob for PipelineAnalysisJob {
    type Diagnosis = PipelineAnalysisReport;

    fn diagnose(self) -> ReportPending<Self::Diagnosis> {
        ReportPending::<Self::Diagnosis> {
            pending_msg: "Analysis of pipelines...".to_string(),
            job: std::thread::spawn(move || {
                if !self.project.jobs_enabled {
                    return self.to_report(
                        vec![ReportStatus::NA("No CI/CD configured on this project".to_string())],
                        vec![]);
                }

                let endpoint = gitlab::api::projects::pipelines::Pipelines::builder()
                    .project(self.project.id)
                    .build()
                    .unwrap();
                let query: Result<Vec<GitlabPipeline>, _> =
                    gitlab::api::paged(endpoint, Pagination::All).query(&self.gitlab);
                match query {
                    Err(e) => {
                        self.to_report(
                            vec![ReportStatus::ERROR(format!("Error : {}", e.to_string()))],
                            vec![]
                        )
                    }
                    Ok(mut pipelines) => {
                        let ref_date = Local::now() - Duration::days(ARTIFACT_JOBS_DAYS_LIMIT);
                        pipelines.sort_by(|a, b| a.created_at.partial_cmp(&b.created_at).unwrap());
                        self.to_report(
                            vec![ReportStatus::NA(
                                format!("{} pipelines. {} pipelines are older than {} days",
                                    pipelines.len(),
                                    pipelines.iter()
                                        .position(|e| e.created_at > ref_date)
                                        .unwrap_or(pipelines.len()),
                                    ARTIFACT_JOBS_DAYS_LIMIT))],
                                    pipelines)
                    }
                }
            }),
            progress: None,
            total: None
        }
    }
}

impl PipelineAnalysisJob {
    pub fn from(gitlab: &GitlabRepository) -> PipelineAnalysisJob {
        PipelineAnalysisJob {
            gitlab: gitlab.gitlab.clone(),
            project: gitlab.project.clone(),
        }
    }
}