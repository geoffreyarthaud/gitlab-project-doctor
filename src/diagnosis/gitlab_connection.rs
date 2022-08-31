use std::env;
use std::error;

use git2::Repository;
use gitlab::api::{projects, Query};
use gitlab::Gitlab;
use human_bytes::human_bytes;
use regex::Regex;
use serde::Deserialize;

use crate::diagnosis::{
    warning_if, ReportJob, ReportPending, ReportStatus, Reportable, ARTIFACT_JOBS_LIMIT,
    PACKAGE_REGISTRY_LIMIT, REPO_LIMIT, STORAGE_LIMIT,
};
use crate::fl;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Deserialize, Clone)]
pub struct Statistics {
    pub commit_count: u64,
    pub storage_size: u64,
    pub repository_size: u64,
    pub job_artifacts_size: u64,
    pub packages_size: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub statistics: Statistics,
    pub jobs_enabled: bool,
}

pub struct GitlabRepository {
    pub url: String,
    pub gitlab: Gitlab,
    pub project: Project,
    pub repo: Option<Repository>,
}

pub struct ConnectionReport {
    pub data: Option<GitlabRepository>,
    pub report_status: Vec<ReportStatus>,
}

pub enum ConnectionJob {
    FromUrl(String),
    FromPath(String),
}

impl ReportJob for ConnectionJob {
    type Diagnosis = ConnectionReport;

    fn diagnose(self) -> ReportPending<Self::Diagnosis> {
        ReportPending::<Self::Diagnosis> {
            pending_msg: fl!("connecting-to-gitlab"),
            job: {
                std::thread::spawn(|| {
                    ConnectionJob::_to_report_status(match self {
                        ConnectionJob::FromUrl(url) => ConnectionJob::_from_url(&url),
                        ConnectionJob::FromPath(path) => ConnectionJob::_from_git_path(&path),
                    })
                })
            },
            progress: None,
            total: None,
        }
    }
}

impl Reportable for ConnectionReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl ConnectionJob {
    fn _to_report_status(result: Result<GitlabRepository>) -> ConnectionReport {
        match result {
            Ok(gitlab) => ConnectionReport {
                report_status: vec![
                    ReportStatus::OK(fl!("gitlab-repo", repo = gitlab.url.as_str())),
                    _report_global_storage(&gitlab.project),
                    _report_repo_storage(&gitlab.project),
                    _report_artifact_storage(&gitlab.project),
                    _report_package_storage(&gitlab.project),
                ],
                data: Some(gitlab),
            },
            Err(e) => ConnectionReport {
                data: None,
                report_status: vec![ReportStatus::ERROR(format!("{}", e))],
            },
        }
    }
    fn _gitlab_project(server: &str, path: &str) -> Result<(Gitlab, Project)> {
        let token = env::var("GL_TOKEN").map_err(|_| fl!("error-gl-token"))?;
        let client = Gitlab::new(server, token)?;
        let endpoint = projects::Project::builder()
            .project(path)
            .statistics(true)
            .build()
            .unwrap();

        let project: Project = endpoint.query(&client)?;
        Ok((client, project))
    }

    fn _from_url(url: &str) -> Result<GitlabRepository> {
        let (server, path) = path_from_git_url(url).ok_or_else(|| fl!("error-not-gitlab-repo"))?;
        let (gitlab, project) = ConnectionJob::_gitlab_project(server, path)?;
        Ok(GitlabRepository {
            url: String::from(path),
            gitlab,
            project,
            repo: None,
        })
    }

    fn _from_git_path(path: &str) -> Result<GitlabRepository> {
        let repo = Repository::open(path).map_err(|_| fl!("error-not-git-repo"))?;
        let (server, url_path) = gitlab_url(&repo).ok_or_else(|| fl!("error-no-gitlab-remote"))?;
        let (gitlab, project) = ConnectionJob::_gitlab_project(&server, &url_path)?;
        Ok(GitlabRepository {
            url: url_path,
            gitlab,
            project,
            repo: Some(repo),
        })
    }
}

fn _report_global_storage(project: &Project) -> ReportStatus {
    let msg = format!(
        "{} {}",
        fl!("size-storage"),
        human_bytes(project.statistics.storage_size as f64)
    );

    warning_if(project.statistics.storage_size > STORAGE_LIMIT, msg)
}

fn _report_repo_storage(project: &Project) -> ReportStatus {
    let msg = format!(
        "{} {} ({} %)",
        fl!("size-git-repo"),
        human_bytes(project.statistics.repository_size as f64),
        100 * project.statistics.repository_size / project.statistics.storage_size
    );

    warning_if(project.statistics.repository_size > REPO_LIMIT, msg)
}

fn _report_artifact_storage(project: &Project) -> ReportStatus {
    let msg = format!(
        "{} {} ({} %)",
        fl!("size-artifacts"),
        human_bytes(project.statistics.job_artifacts_size as f64),
        100 * project.statistics.job_artifacts_size / project.statistics.storage_size
    );
    warning_if(
        project.statistics.job_artifacts_size > ARTIFACT_JOBS_LIMIT,
        msg,
    )
}

fn _report_package_storage(project: &Project) -> ReportStatus {
    let msg = format!(
        "{} {} ({} %)",
        fl!("size-packages"),
        human_bytes(project.statistics.packages_size as f64),
        100 * project.statistics.packages_size / project.statistics.storage_size
    );
    warning_if(
        project.statistics.packages_size > PACKAGE_REGISTRY_LIMIT,
        msg,
    )
}

fn gitlab_url(repo: &Repository) -> Option<(String, String)> {
    let full_url = repo
        .remotes()
        .unwrap()
        .iter()
        .filter(|rmt_name| rmt_name.is_some())
        .map(|rmt_name| {
            let remote = repo.find_remote(rmt_name.unwrap()).unwrap();
            String::from(remote.url().unwrap())
        })
        .find(move |_| true)?;
    let (server, path) = path_from_git_url(&full_url)?;
    Some((String::from(server), String::from(path)))
}

fn path_from_git_url(url: &str) -> Option<(&str, &str)> {
    _path_from_https_url(url).or_else(|| _path_from_ssh_url(url))
}

fn _path_from_https_url(url: &str) -> Option<(&str, &str)> {
    let regex_git = Regex::new("(http(s)?://)(.+?)/(.+)(\\.git)?(/)?").unwrap();
    let caps = regex_git.captures(url);
    let server = caps.as_ref()?.get(3)?.as_str();
    let path = caps.as_ref()?.get(4)?.as_str();
    Some((server, path.trim_end_matches('/').trim_end_matches(".git")))
}

fn _path_from_ssh_url(url: &str) -> Option<(&str, &str)> {
    let regex_git = Regex::new("(git@)(.+?):(.+)(\\.git)(/)?").unwrap();
    let caps = regex_git.captures(url);
    let server = caps.as_ref()?.get(2)?.as_str();
    let path = caps.as_ref()?.get(3)?.as_str();
    Some((server, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_from_ministry_https_url() {
        // GIVEN
        let url =
            "https://gitlab-forge.din.developpement-durable.gouv.fr/snum/dam/gitlab/gitlab-usage.git";
        // WHEN
        let path = path_from_git_url(url);
        // THEN
        assert!(path.is_some());
        assert_eq!(
            (
                "gitlab-forge.din.developpement-durable.gouv.fr",
                "snum/dam/gitlab/gitlab-usage"
            ),
            path.unwrap()
        );
    }

    #[test]
    fn path_from_ministry_https_url_without_git() {
        // GIVEN
        let url =
            "https://gitlab-forge.din.developpement-durable.gouv.fr/snum/dam/gitlab/gitlab-usage";
        // WHEN
        let path = path_from_git_url(url);
        // THEN
        assert!(path.is_some());
        assert_eq!(
            (
                "gitlab-forge.din.developpement-durable.gouv.fr",
                "snum/dam/gitlab/gitlab-usage"
            ),
            path.unwrap()
        );
    }

    #[test]
    fn path_from_gitlab_https_url() {
        // GIVEN
        let url = "https://gitlab.com/visiplus.formateur/debuter-javascript.git";
        // WHEN
        let path = path_from_git_url(url);
        // THEN
        assert!(path.is_some());
        assert_eq!(
            ("gitlab.com", "visiplus.formateur/debuter-javascript"),
            path.unwrap()
        );
    }

    #[test]
    fn path_from_gitlab_ssh_url() {
        // GIVEN
        let url = "git@gitlab.com:visiplus.formateur/debuter-javascript.git";
        // WHEN
        let path = path_from_git_url(url);
        // THEN
        assert!(path.is_some());
        assert_eq!(
            ("gitlab.com", "visiplus.formateur/debuter-javascript"),
            path.unwrap()
        );
    }
}
