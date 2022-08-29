use std::cmp::max;
use std::sync::mpsc;

use chrono::{Duration, Local};
use gitlab::api::{ApiError, Query};
use gitlab::Gitlab;
use human_bytes::human_bytes;

use crate::diagnosis::gitlab_connection::Project;
use crate::diagnosis::pipeline_analysis::{GitlabPipeline, PipelineAnalysisReport};
use crate::diagnosis::{RemedyJob, GITLAB_SCOPE_ERROR};
use crate::{ReportPending, ReportStatus, Reportable};

pub struct PipelineCleanJob {
    pub pipeline_report: PipelineAnalysisReport,
    pub days: i64,
}

pub struct PipelineCleanReport {
    pub saved_bytes: u64,
    pub deleted_pipelines: Vec<GitlabPipeline>,
    pub report_status: Vec<ReportStatus>,
}

impl Reportable for PipelineCleanReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl PipelineCleanReport {
    fn fatal_error(id: u64, msg: &str) -> Self {
        Self {
            saved_bytes: 0,
            deleted_pipelines: vec![],
            report_status: vec![ReportStatus::ERROR(format!(
                "Pipeline {} - Error : {}",
                id, msg
            ))],
        }
    }
}

impl RemedyJob for PipelineCleanJob {
    type Report = PipelineCleanReport;

    fn remedy(self) -> ReportPending<Self::Report> {
        let (tx, rx) = mpsc::channel();
        let ref_date = Local::now() - Duration::days(self.days);
        let count = self
            .pipeline_report
            .pipelines
            .iter()
            .filter(|a| a.created_at <= ref_date)
            .count();
        ReportPending {
            pending_msg: "Deleting old pipelines".to_string(),
            job: std::thread::spawn(move || {
                let mut deleted_pipelines = vec![];

                for (i, pipeline) in self.pipeline_report.pipelines.into_iter().enumerate() {
                    if pipeline.created_at > ref_date {
                        break;
                    }
                    let mut retry = 0;
                    loop {
                        let endpoint = gitlab::api::projects::pipelines::DeletePipeline::builder()
                            .project(self.pipeline_report.project.id)
                            .pipeline(pipeline.id)
                            .build()
                            .unwrap();
                        let query =
                            gitlab::api::ignore(endpoint).query(&self.pipeline_report.gitlab);
                        match query {
                            Ok(_) => {
                                deleted_pipelines.push(pipeline);
                                break;
                            }
                            Err(e) => match e {
                                ApiError::Gitlab { msg } => {
                                    return match msg.as_str() {
                                            msg if msg.contains(GITLAB_SCOPE_ERROR) => {
                                                PipelineCleanReport::fatal_error(
                                                    pipeline.id,
                                                    "Your token has insufficient privileges to delete pipelines")
                                            }
                                            other => {
                                                PipelineCleanReport::fatal_error(
                                                    pipeline.id,
                                                    other)
                                            }
                                        };
                                }
                                ApiError::Client { source } => {
                                    retry += 1;
                                    if retry >= 3 {
                                        return PipelineCleanReport::fatal_error(
                                            pipeline.id,
                                            source.to_string().as_str(),
                                        );
                                    }
                                }
                                _ => {
                                    return PipelineCleanReport::fatal_error(
                                        pipeline.id,
                                        e.to_string().as_str(),
                                    );
                                }
                            },
                        }
                    }
                    let _ = tx.send(i);
                }
                let saved_bytes = PipelineCleanJob::_compute_saved_bytes(
                    &self.pipeline_report.gitlab,
                    &self.pipeline_report.project,
                );
                PipelineCleanReport {
                    saved_bytes,
                    report_status: vec![ReportStatus::OK(format!(
                        "Deleted {} pipelines, {} saved.",
                        deleted_pipelines.len(),
                        human_bytes(saved_bytes as f64)
                    ))],
                    deleted_pipelines,
                }
            }),
            progress: Some(rx),
            total: Some(count),
        }
    }
}

impl PipelineCleanJob {
    pub fn from(pipeline_report: PipelineAnalysisReport, days: i64) -> Self {
        if days < 0 {
            panic!("Number of days must be 1 or superior")
        }
        Self {
            pipeline_report,
            days,
        }
    }

    fn _compute_saved_bytes(gitlab: &Gitlab, project: &Project) -> u64 {
        let old_size = project.statistics.job_artifacts_size;
        let endpoint = gitlab::api::projects::Project::builder()
            .project(project.id)
            .statistics(true)
            .build()
            .unwrap();

        let new_size = endpoint
            .query(gitlab)
            .map(|p: Project| p.statistics.job_artifacts_size)
            .unwrap_or(old_size);
        max(0, old_size - new_size)
    }
}
