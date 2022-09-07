use structopt::StructOpt;

use cli::Args;

use crate::diagnosis::gitlab_connection::{ConnectionJob, GitlabRepository, Statistics};
use crate::diagnosis::job_analysis::{JobAnalysisJob, JobAnalysisReport};
use crate::diagnosis::package_analysis::{PackageAnalysisJob, PackageAnalysisReport};
use crate::diagnosis::package_clean::PackageCleanJob;
use crate::diagnosis::pipeline_analysis::{PipelineAnalysisJob, PipelineAnalysisReport};
use crate::diagnosis::pipeline_clean::PipelineCleanJob;
use crate::diagnosis::ReportStatus;
use crate::diagnosis::{RemedyJob, ReportJob, ReportPending, Reportable};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use serde::Serialize;

pub mod api;
pub mod cli;
pub mod diagnosis;

// --- Code boilerplate to load i18n resources
#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

lazy_static! {
    pub static ref LANGUAGE_LOADER: FluentLanguageLoader = {
        let language_loader: FluentLanguageLoader = fluent_language_loader!();
        let requested_languages = DesktopLanguageRequester::requested_languages();
        let _result = i18n_embed::select(&language_loader, &Localizations, &requested_languages);
        language_loader
    };
}

#[macro_export(local_inner_macros)]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::LANGUAGE_LOADER, $message_id, $($args) *)
    }};
}
// --- End of code boilerplate to load i18n resources

#[derive(Serialize)]
struct AnalysisReport {
    pub url: String,
    pub stats: Statistics,
    pub savable_bytes_jobs: u64,
    pub savable_bytes_packages: u64,
    pub savable_bytes_containers: u64,
    pub rating: Option<String>,
}

impl AnalysisReport {
    pub fn compute_rating(&mut self) {
        self.rating = Some("A".to_string());
    }
}

fn _connect_to_gitlab(args: &Args) -> GitlabRepository {
    let connection_job = {
        if args.url.is_some() {
            ConnectionJob::FromUrl(args.url.as_ref().unwrap().clone())
        } else {
            let default_path = String::from(".");
            let path: &str = args.git_path.as_ref().unwrap_or(&default_path);
            ConnectionJob::FromPath(path.to_string())
        }
    };

    // Connection to Gitlab
    let report_pending = connection_job.diagnose();

    let connection = cli::display_report_pending(report_pending);

    cli::fatal_if_none(connection.data, "Diagnosis stops here.")
}

fn _analyze_pipelines(days: usize, connection_data: &GitlabRepository) -> PipelineAnalysisReport {
    let report_pending = PipelineAnalysisJob::from(connection_data, days).diagnose();
    cli::display_report_pending(report_pending)
}

fn _clean_pipelines(days: usize, analysis_report: PipelineAnalysisReport) {
    let report_pending = PipelineCleanJob::from(analysis_report, days).remedy();
    let _ = cli::display_report_pending(report_pending);
}

fn _analyze_jobs(days: usize, connection_data: &GitlabRepository) -> JobAnalysisReport {
    let report_pending = JobAnalysisJob::from(connection_data, days).diagnose();
    cli::display_report_pending(report_pending)
}

fn _analyse_packages(connection_data: &GitlabRepository) -> PackageAnalysisReport {
    let report_pending = PackageAnalysisJob::from(connection_data).diagnose();
    cli::display_report_pending(report_pending)
}

fn _clean_packages(report: PackageAnalysisReport) {
    if !report.obsolete_files.is_empty() {
        return;
    }
    let report_pending = PackageCleanJob::from(report).remedy();
    let _ = cli::display_report_pending(report_pending);
}

fn main() {
    let args = Args::from_args();
    eprintln!("Gitlab Project Doctor v{}", env!("CARGO_PKG_VERSION"));
    let connection_data = _connect_to_gitlab(&args);
    if args.analysis_mode {
        // Analysis mode

        let job_report = _analyze_jobs(args.days, &connection_data);
        let package_report = _analyse_packages(&connection_data);
        let mut global_report = AnalysisReport {
            url: connection_data.url,
            stats: connection_data.project.statistics,
            savable_bytes_jobs: job_report.savable_bytes,
            savable_bytes_packages: package_report.savable_bytes,
            savable_bytes_containers: 0,
            rating: None,
        };
        global_report.compute_rating();
        println!("{}", serde_json::to_string(&global_report).unwrap());
    } else if args.batch_mode {
        // Batch mode

        _clean_pipelines(args.days, _analyze_pipelines(args.days, &connection_data));
        _clean_packages(_analyse_packages(&connection_data));
    } else {
        // Interactive mode

        let report = _analyze_pipelines(args.days, &connection_data);
        match cli::input_clean_artifacts(args.days) {
            None => {
                cli::console_report_statuses(
                    &[ReportStatus::WARNING(
                        fl!("pipeline-no-deletion").to_string(),
                    )],
                    2,
                );
            }
            Some(days) => {
                _clean_pipelines(days, report);
            }
        }
        let report = _analyse_packages(&connection_data);
        if cli::input_clean_files() {
            _clean_packages(report);
        } else {
            cli::console_report_statuses(
                &[ReportStatus::WARNING(
                    fl!("package-no-deletion").to_string(),
                )],
                2,
            );
        }
    }
}
