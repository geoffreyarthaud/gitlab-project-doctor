use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
use structopt::StructOpt;

use cli::Args;

use crate::diagnosis::{Reportable, ReportJob, ReportPending};
use crate::diagnosis::artifact_size::ArtifactSizeJob;
use crate::diagnosis::gitlab_connection::ConnectionJob;
use crate::diagnosis::ReportStatus;

pub mod diagnosis;
pub mod cli;

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
    let report_pending = ArtifactSizeJob::from(&connection_data).diagnose();
    let _ = cli::display_report_pending(report_pending);

}
