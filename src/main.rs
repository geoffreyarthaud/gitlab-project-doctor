use structopt::StructOpt;

use cli::Args;

use crate::diagnosis::{RemedyJob, Reportable, ReportJob, ReportPending};
use crate::diagnosis::gitlab_connection::ConnectionJob;
use crate::diagnosis::package_analysis::PackageAnalysisJob;
use crate::diagnosis::pipeline_analysis::PipelineAnalysisJob;
use crate::diagnosis::pipeline_clean::PipelineCleanJob;
use crate::diagnosis::ReportStatus;

pub mod diagnosis;
pub mod cli;
pub mod api;

fn main() {
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
    let report_pending = PipelineAnalysisJob::from(&connection_data).diagnose();
    let pipeline_report = cli::display_report_pending(report_pending);
    if !pipeline_report.pipelines.is_empty() {
        if let Some(days) = cli::input_clean_artifacts() {
            let report_pending = PipelineCleanJob::from(pipeline_report, days).remedy();
            let _ = cli::display_report_pending(report_pending);
        } else {
            cli::console_report_statuses(
                &[ReportStatus::WARNING("Jobs deletion cancelled".to_string())],
                2);
        }
    }

    // Analysis of packages
    let report_pending = PackageAnalysisJob::from(&connection_data).diagnose();
    let report = cli::display_report_pending(report_pending);
    eprintln!("{:?}", report.obsolete_files);
}
