use chrono::{DateTime, Duration, Local};
use gitlab::api::{Pagination, Query};
use gitlab::Gitlab;
use human_bytes::human_bytes;
use serde::Deserialize;

use crate::diagnosis::gitlab_connection::{GitlabRepository, Project};
use crate::diagnosis::{warning_if, CONTAINER_REGISTRY_LIMIT};
use crate::{api, fl, ReportJob, ReportPending, ReportStatus, Reportable};

#[derive(Debug, Deserialize)]
pub struct GitlabRawContainerRepository {
    pub id: u64,
    pub created_at: DateTime<Local>,
    pub tags: Vec<GitlabContainerTagSummary>,
}

#[derive(Debug, Deserialize)]
pub struct GitlabContainerRepository {
    pub id: u64,
    pub created_at: DateTime<Local>,
    pub tags: Vec<GitlabContainerTag>,
}

#[derive(Debug, Deserialize)]
pub struct GitlabContainerTagSummary {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GitlabContainerTag {
    pub name: String,
    pub created_at: DateTime<Local>,
    pub total_size: u64,
}

pub struct ContainerAnalysisJob {
    pub gitlab: Gitlab,
    pub project: Project,
    pub days: usize,
}

pub struct ContainerAnalysisReport {
    pub gitlab: Gitlab,
    pub project: Project,
    pub containers: Vec<GitlabContainerRepository>,
    pub report_status: Vec<ReportStatus>,
}

impl Reportable for ContainerAnalysisReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl ContainerAnalysisJob {
    fn to_report(
        self,
        report_status: Vec<ReportStatus>,
        containers: Vec<GitlabContainerRepository>,
    ) -> ContainerAnalysisReport {
        ContainerAnalysisReport {
            gitlab: self.gitlab,
            project: self.project,
            containers,
            report_status,
        }
    }

    fn get_detailed_repo(
        &self,
        containers: &[GitlabRawContainerRepository],
    ) -> Vec<GitlabContainerRepository> {
        containers
            .iter()
            .map(|cr| GitlabContainerRepository {
                id: cr.id,
                created_at: cr.created_at,
                tags: cr
                    .tags
                    .iter()
                    .map(|t| self.get_detailed_tag(t, cr.id))
                    .collect(),
            })
            .collect()
    }

    fn get_detailed_tag(
        &self,
        tag: &GitlabContainerTagSummary,
        repo_id: u64,
    ) -> GitlabContainerTag {
        let endpoint = api::registry::Tag::builder()
            .project(self.project.id)
            .repository(repo_id)
            .tag_name(tag.name.clone())
            .build()
            .unwrap();
        endpoint.query(&self.gitlab).unwrap()
    }
}

impl ReportJob for ContainerAnalysisJob {
    type Diagnosis = ContainerAnalysisReport;

    fn diagnose(self) -> ReportPending<Self::Diagnosis> {
        ReportPending::<Self::Diagnosis> {
            pending_msg: fl!("container-analysing"),
            job: std::thread::spawn(move || {
                if !self.project.jobs_enabled {
                    return self.to_report(vec![ReportStatus::NA(fl!("no-cicd"))], vec![]);
                }

                let endpoint = api::registry::Repositories::builder()
                    .project(self.project.id)
                    .tags(true)
                    .build()
                    .unwrap();
                let query: Result<Vec<GitlabRawContainerRepository>, _> =
                    gitlab::api::paged(endpoint, Pagination::All).query(&self.gitlab);
                match query {
                    Err(e) => self.to_report(
                        vec![ReportStatus::ERROR(format!(
                            "{} {}",
                            fl!("error"),
                            e.to_string()
                        ))],
                        vec![],
                    ),
                    Ok(containers) => {
                        let container_repos = self.get_detailed_repo(&containers);
                        let days = self.days;
                        let ref_date = Local::now() - Duration::days(days as i64);
                        let image_count: usize =
                            container_repos.iter().map(|cr| cr.tags.len()).sum();
                        let registry_size: u64 = container_repos
                            .iter()
                            .map(|cr| {
                                let res: u64 = cr.tags.iter().map(|t| t.total_size).sum();
                                res
                            })
                            .sum();
                        let old_image_count: usize = container_repos
                            .iter()
                            .map(|cr| cr.tags.iter().filter(|t| t.created_at < ref_date).count())
                            .sum();
                        self.to_report(
                            vec![
                                warning_if(
                                    registry_size > CONTAINER_REGISTRY_LIMIT,
                                    fl!(
                                        "container-summary",
                                        registry_size = human_bytes(registry_size as f64)
                                    ),
                                ),
                                ReportStatus::NA(fl!(
                                    "container-report",
                                    image_count = image_count,
                                    old_image_count = old_image_count,
                                    nb_days = days
                                )),
                            ],
                            container_repos,
                        )
                    }
                }
            }),
            progress: None,
            total: None,
        }
    }
}

impl ContainerAnalysisJob {
    pub fn from(gitlab: &GitlabRepository, days: usize) -> ContainerAnalysisJob {
        ContainerAnalysisJob {
            gitlab: gitlab.gitlab.clone(),
            project: gitlab.project.clone(),
            days,
        }
    }
}
