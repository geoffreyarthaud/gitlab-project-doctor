use std::cmp::max;
use std::sync::mpsc;

use chrono::{Duration, Local};
use gitlab::api::{ApiError, Query};
use gitlab::Gitlab;
use human_bytes::human_bytes;

use crate::diagnosis::gitlab_connection::Project;
use crate::diagnosis::pipeline_analysis::{GitlabPipeline, PipelineAnalysisReport};
use crate::diagnosis::{RemedyJob, GITLAB_SCOPE_ERROR};
use crate::{fl, ReportPending, ReportStatus, Reportable};

pub struct PipelineCleanJob {
    pub pipeline_report: PipelineAnalysisReport,
    pub days: usize,
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
        let ref_date = Local::now() - Duration::days(self.days as i64);
        let count = self
            .pipeline_report
            .pipelines
            .iter()
            .filter(|a| a.created_at <= ref_date)
            .count();
        ReportPending {
            pending_msg: fl!("pipeline-deleting"),
            job: std::thread::spawn(move || {
                let mut deleted_pipelines = vec![];
                let last_index = self.pipeline_report.pipelines.len() - 1;
                let mut last_is_old = false;
                for (i, pipeline) in self.pipeline_report.pipelines.into_iter().enumerate() {
                    if pipeline.created_at > ref_date {
                        break;
                    }
                    if i == last_index {
                        last_is_old = true;
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
                                                &fl!("error-insufficient-privileges"),
                                            )
                                        }
                                        other => {
                                            PipelineCleanReport::fatal_error(pipeline.id, other)
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
                let mut report_status = vec![ReportStatus::OK(fl!(
                    "pipeline-clean-report",
                    nb_pipelines = deleted_pipelines.len(),
                    size = human_bytes(saved_bytes as f64)
                ))];
                if last_is_old {
                    report_status.push(ReportStatus::NA(fl!("pipeline-last-notdeleted")));
                }
                PipelineCleanReport {
                    saved_bytes,
                    report_status,
                    deleted_pipelines,
                }
            }),
            progress: Some(rx),
            total: Some(count),
        }
    }
}

impl PipelineCleanJob {
    pub fn from(pipeline_report: PipelineAnalysisReport, days: usize) -> Self {
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
