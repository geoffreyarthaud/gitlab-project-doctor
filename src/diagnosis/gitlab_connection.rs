use git2::Repository;
use gitlab::api::{projects, Query};
use gitlab::Gitlab;
use serde::Deserialize;
use std::env;
use std::error;

use crate::diagnosis::{Diagnosis, Report, ReportStatus};

const FORGE_URL: &str = "gitlab-forge.din.developpement-durable.gouv.fr";

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Deserialize)]
pub struct Statistics {
    pub commit_count: u64,
    pub storage_size: u64,
    pub repository_size: u64,
    pub job_artifacts_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub statistics: Statistics,
}

pub struct GitlabRepository {
    pub url: String,
    pub gitlab: Gitlab,
    pub project: Project,
    pub repo: Option<Repository>,
}

pub struct GitlabConnection {
    pub data: Option<GitlabRepository>,
    pub report: Report,
}

impl Diagnosis for GitlabConnection {
    fn diagnosis(&mut self) -> &Report {
        &self.report
    }
}

impl GitlabConnection {
    pub fn from_path(path: &str) -> GitlabConnection {
        match GitlabConnection::_from_path(path) {
            Ok(gitlab) => GitlabConnection {
                report: Report {
                    global: ReportStatus::OK(format!("Gitlab repository : {}", gitlab.url)),
                    details: vec![],
                },
                data: Some(gitlab),
            },
            Err(e) => GitlabConnection {
                data: None,
                report: Report {
                    global: ReportStatus::ERROR(format!("{}", e)),
                    details: vec![],
                },
            },
        }
    }
    fn _from_path(path: &str) -> Result<GitlabRepository> {
        let repo = Repository::open(path).or_else(|_| Err("This dir is not a Git repository"))?;
        let url = gitlab_url(&repo).ok_or("This dir does not contain a gitlab remote")?;
        let token = env::var("GL_TOKEN").or_else(|_| {
            Err("GL_TOKEN environment variable must contain a valid Gitlab private token")
        })?;
        let client = Gitlab::new(FORGE_URL, token)?;
        let endpoint = projects::Project::builder()
            .project(url.clone())
            .statistics(true)
            .build()
            .unwrap();

        let project: Project = endpoint.query(&client)?;
        Ok(GitlabRepository {
            url: url,
            gitlab: client,
            project: project,
            repo: Some(repo),
        })
    }
}

fn gitlab_url(repo: &git2::Repository) -> Option<String> {
    repo.remotes()
        .unwrap()
        .iter()
        .filter(|rmt_name| rmt_name.is_some())
        .map(|rmt_name| {
            let remote = repo.find_remote(rmt_name.unwrap()).unwrap();
            String::from(remote.url().unwrap())
        })
        .find(move |url| url.find(FORGE_URL).is_some())
        .map(|url| {
            // Remove gitlab base URL + "/"
            let pos = url.find(FORGE_URL).unwrap() + FORGE_URL.len() + 1;
            // Remove ".git"
            let end_pos = url.rfind(".git").unwrap_or(url.len());
            String::from(url.get(pos..end_pos).unwrap())
        })
}
