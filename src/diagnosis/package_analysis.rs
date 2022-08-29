use chrono::{DateTime, Local};
use gitlab::api::{Pagination, Query};
use gitlab::Gitlab;
use human_bytes::human_bytes;
use serde::Deserialize;

use crate::{Reportable, ReportJob, ReportPending, ReportStatus};
use crate::api::packages::{PackageFiles, Packages};
use crate::diagnosis::gitlab_connection::{GitlabRepository, Project};

#[derive(Debug, Deserialize)]
pub struct GitlabPackage {
    pub id: u64,
    pub name: String,
    pub package_type: String,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Deserialize)]
pub struct GitlabPackageFile {
    pub id: u64,
    pub created_at: DateTime<Local>,
    pub file_name: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct PackageWithFile {
    pub package: GitlabPackage,
    pub sorted_files: Vec<GitlabPackageFile>,
}

pub struct PackageAnalysisJob {
    pub gitlab: Gitlab,
    pub project: Project,
}

pub struct PackageAnalysisReport {
    pub gitlab: Gitlab,
    pub project: Project,
    pub packages: Vec<PackageWithFile>,
    pub report_status: Vec<ReportStatus>,
    pub savable_files: usize,
    pub savable_bytes: u64,
}

impl Reportable for PackageAnalysisReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl PackageAnalysisJob {
    pub fn default_report(self, status: ReportStatus) -> PackageAnalysisReport {
        PackageAnalysisReport {
            gitlab: self.gitlab,
            project: self.project,
            packages: vec![],
            report_status: vec![status],
            savable_files: 0,
            savable_bytes: 0,
        }
    }
}

impl ReportJob for PackageAnalysisJob {
    type Diagnosis = PackageAnalysisReport;

    fn diagnose(self) -> ReportPending<Self::Diagnosis> {
        ReportPending::<Self::Diagnosis> {
            pending_msg: "Analysis of packages...".to_string(),
            job: std::thread::spawn(move || {
                if !self.project.jobs_enabled {
                    return self.default_report(
                        ReportStatus::NA("No CI/CD configured on this project".to_string()));
                }

                let endpoint = Packages::builder()
                    .project(self.project.id)
                    .build()
                    .unwrap();
                let query: Result<Vec<GitlabPackage>, _> =
                    gitlab::api::paged(endpoint, Pagination::All).query(&self.gitlab);
                match query {
                    Err(e) => {
                        self.default_report(
                            ReportStatus::ERROR(format!("Error : {}", e.to_string())))
                    }
                    Ok(mut packages) => {
                        packages.sort_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap());
                        let mut savable_bytes = 0;
                        let mut savable_files = 0;
                        let mut packages_with_files = vec![];
                        for package in packages.into_iter() {
                            let endpoint = PackageFiles::builder()
                                .project(self.project.id)
                                .package(package.id)
                                .build()
                                .unwrap();
                            let mut files: Vec<GitlabPackageFile> =
                                gitlab::api::paged(endpoint, Pagination::All)
                                    .query(&self.gitlab)
                                    .unwrap();
                            files.sort_by(|a, b| a.created_at.partial_cmp(&b.created_at).unwrap());
                            let obsolete_ids = detect_obsolete_files(&package, &files);

                            savable_files += obsolete_ids.len();
                            savable_bytes += obsolete_ids.into_iter()
                                .map(|i| files[i].size)
                                .sum::<u64>();

                            packages_with_files.push(PackageWithFile {
                                package,
                                sorted_files: files,
                            })
                        }
                        PackageAnalysisReport {
                            gitlab: self.gitlab,
                            project: self.project,
                            report_status: vec![ReportStatus::NA(
                                format!("{} packages. {} files are duplicated ({})",
                                        packages_with_files.len(),
                                        savable_files,
                                        human_bytes(savable_bytes as f64)))],
                            packages: packages_with_files,
                            savable_files,
                            savable_bytes,
                        }
                    }
                }
            }),
            progress: None,
            total: None,
        }
    }
}

impl PackageAnalysisJob {
    pub fn from(gitlab: &GitlabRepository) -> PackageAnalysisJob {
        PackageAnalysisJob {
            gitlab: gitlab.gitlab.clone(),
            project: gitlab.project.clone(),
        }
    }
}

impl PackageAnalysisReport {}

fn detect_obsolete_files(_: &GitlabPackage,
                         sorted_files: &[GitlabPackageFile]) -> Vec<usize> {
    let mut ids = vec![];
    let mut names : Vec<&str> = vec![];
    for (idx, file) in sorted_files.iter().enumerate() {
        if names.iter().any(|e| **e == file.file_name) {
            ids.push(idx);
        } else {
            names.push(&file.file_name);
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn detect_generic_obsolete_files_nominal() {
        // GIVEN
        let package = GitlabPackage {
            id: 42,
            name: "generic".to_string(),
            package_type: "".to_string(),
            created_at: Local::now() - Duration::days(30),
        };
        let files = vec![GitlabPackageFile {
            id: 50,
            created_at: Local::now() - Duration::days(4),
            file_name: "abc.txt".to_string(),
            size: 13,
        }, GitlabPackageFile {
            id: 54,
            created_at: Local::now() - Duration::days(5),
            file_name: "abc.txt".to_string(),
            size: 13,
        }, GitlabPackageFile {
            id: 50,
            created_at: Local::now() - Duration::days(6),
            file_name: "zyx.txt".to_string(),
            size: 13,
        },
                         GitlabPackageFile {
                             id: 56,
                             created_at: Local::now() - Duration::days(7),
                             file_name: "abc.txt".to_string(),
                             size: 13,
                         }, GitlabPackageFile {
                id: 50,
                created_at: Local::now() - Duration::days(8),
                file_name: "zyx.txt".to_string(),
                size: 13,
            }];

        // WHEN
        let ids = detect_obsolete_files(&package, &files);

        // THEN
        assert_eq!(ids, vec![1, 3, 4]);
    }
}