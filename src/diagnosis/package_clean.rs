use std::sync::mpsc;

use gitlab::api::{ApiError, Query};
use human_bytes::human_bytes;

use crate::diagnosis::package_analysis::{FileFromPackage, PackageAnalysisReport};
use crate::diagnosis::{RemedyJob, GITLAB_SCOPE_ERROR};
use crate::{api, fl, ReportPending, ReportStatus, Reportable};

pub struct PackageCleanJob {
    pub package_report: PackageAnalysisReport,
}

pub struct PackageCleanReport {
    pub saved_bytes: u64,
    pub deleted_files: Vec<FileFromPackage>,
    pub report_status: Vec<ReportStatus>,
}

impl Reportable for PackageCleanReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl PackageCleanReport {
    fn fatal_error(id: u64, msg: &str) -> Self {
        Self {
            saved_bytes: 0,
            deleted_files: vec![],
            report_status: vec![ReportStatus::ERROR(format!(
                "Package {} - {} {}",
                id,
                fl!("error"),
                msg
            ))],
        }
    }
}

impl RemedyJob for PackageCleanJob {
    type Report = PackageCleanReport;

    fn remedy(self) -> ReportPending<Self::Report> {
        let (tx, rx) = mpsc::channel();
        let count = self.package_report.obsolete_files.len();

        ReportPending {
            pending_msg: fl!("package-deleting"),
            job: std::thread::spawn(move || {
                let mut deleted_packages_files = vec![];

                for (i, file) in self.package_report.obsolete_files.into_iter().enumerate() {
                    let mut retry = 0;
                    loop {
                        let endpoint = api::packages::DeletePackageFile::builder()
                            .project(self.package_report.project.id)
                            .package(file.package_id)
                            .file(file.file.id)
                            .build()
                            .unwrap();
                        let query =
                            gitlab::api::ignore(endpoint).query(&self.package_report.gitlab);
                        match query {
                            Ok(_) => {
                                deleted_packages_files.push(file);
                                break;
                            }
                            Err(e) => match e {
                                ApiError::Gitlab { msg } => {
                                    return match msg.as_str() {
                                        msg if msg.contains(GITLAB_SCOPE_ERROR) => {
                                            PackageCleanReport::fatal_error(
                                                file.file.id,
                                                &fl!("error-insufficient-privileges"),
                                            )
                                        }
                                        other => {
                                            PackageCleanReport::fatal_error(file.file.id, other)
                                        }
                                    };
                                }
                                ApiError::Client { source } => {
                                    retry += 1;
                                    if retry >= 3 {
                                        return PackageCleanReport::fatal_error(
                                            file.file.id,
                                            source.to_string().as_str(),
                                        );
                                    }
                                }
                                _ => {
                                    return PackageCleanReport::fatal_error(
                                        file.file.id,
                                        e.to_string().as_str(),
                                    );
                                }
                            },
                        }
                    }
                    let _ = tx.send(i);
                }
                let saved_bytes = deleted_packages_files.iter().map(|f| f.file.size).sum();
                PackageCleanReport {
                    saved_bytes,
                    report_status: vec![ReportStatus::OK(fl!(
                        "package-clean-report",
                        nb_packages = deleted_packages_files.len(),
                        size = human_bytes(saved_bytes as f64)
                    ))],
                    deleted_files: deleted_packages_files,
                }
            }),
            progress: Some(rx),
            total: Some(count),
        }
    }
}

impl PackageCleanJob {
    pub fn from(package_report: PackageAnalysisReport) -> Self {
        Self { package_report }
    }
}
