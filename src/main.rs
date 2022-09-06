use structopt::StructOpt;

use cli::Args;

use crate::diagnosis::gitlab_connection::ConnectionJob;
use crate::diagnosis::package_analysis::PackageAnalysisJob;
use crate::diagnosis::package_clean::PackageCleanJob;
use crate::diagnosis::pipeline_analysis::PipelineAnalysisJob;
use crate::diagnosis::pipeline_clean::PipelineCleanJob;
use crate::diagnosis::ReportStatus;
use crate::diagnosis::{RemedyJob, ReportJob, ReportPending, Reportable};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;

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

fn main() {
    eprintln!("Gitlab Project Doctor v{}", env!("CARGO_PKG_VERSION"));
    let args = Args::from_args();

    let connection_job = {
        if args.url.is_some() {
            ConnectionJob::FromUrl(args.url.unwrap())
        } else {
            let default_path = String::from(".");
            let path: &str = args.git_path.as_ref().unwrap_or(&default_path);
            ConnectionJob::FromPath(path.to_string())
        }
    };

    // Connection to Gitlab
    let report_pending = connection_job.diagnose();

    let connection = cli::display_report_pending(report_pending);

    let connection_data = cli::fatal_if_none(connection.data, "Diagnosis stops here.");

    // Analysis of artifacts
    let report_pending = PipelineAnalysisJob::from(&connection_data, args.days).diagnose();
    let pipeline_report = cli::display_report_pending(report_pending);
    if !pipeline_report.pipelines.is_empty() {
        if args.batch_mode {
            let report_pending = PipelineCleanJob::from(pipeline_report, args.days).remedy();
            let _ = cli::display_report_pending(report_pending);
        } else if let Some(days) = cli::input_clean_artifacts(args.days) {
            let report_pending = PipelineCleanJob::from(pipeline_report, days).remedy();
            let _ = cli::display_report_pending(report_pending);
        } else {
            cli::console_report_statuses(
                &[ReportStatus::WARNING("Jobs deletion cancelled".to_string())],
                2,
            );
        }
    }

    // Analysis of packages
    let report_pending = PackageAnalysisJob::from(&connection_data).diagnose();
    let package_report = cli::display_report_pending(report_pending);
    if !package_report.obsolete_files.is_empty() {
        if args.batch_mode || cli::input_clean_files() {
            let report_pending = PackageCleanJob::from(package_report).remedy();
            let _ = cli::display_report_pending(report_pending);
        } else {
            cli::console_report_statuses(
                &[ReportStatus::WARNING(
                    "Files deletion cancelled".to_string(),
                )],
                2,
            );
        }
    }
}
