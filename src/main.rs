use std::process;
use std::fmt::Write as _;
use std::time::Duration;

use console::style;
use indicatif::ProgressBar;
use structopt::StructOpt;

use crate::diagnosis::{Reportable, ReportJob, ReportPending};
use crate::diagnosis::artifact_size::ArtifactSizeJob;
use crate::diagnosis::gitlab_connection::ConnectionJob;
use crate::diagnosis::ReportStatus;

pub mod diagnosis;

fn fatal_if_none<T>(result: Option<T>, msg: &str) -> T {
    match result {
        Some(x) => x,
        None => {
            eprintln!("{}", msg);
            process::exit(1);
        }
    }
}

#[derive(StructOpt)]
struct Args {
    #[structopt(name = "url", long)]
    /// Analyze the project from the URL of Gitlab repository
    url: Option<String>,
    #[structopt(name = "git_path")]
    /// Analyze the project from a local path of a Git repository
    git_path: Option<String>,
}

fn console_report_status(buffer: &mut String, report_status: &ReportStatus, indent: usize) {
    let width = indent + 4;
    let _ = match &report_status {
        ReportStatus::OK(msg) => {
            writeln!(buffer, "{:>width$} {}", style("[✓]").green(), msg, width = width)
        }
        ReportStatus::WARNING(msg) => {
            writeln!(buffer,
                     "{:>width$} {}",
                     style("[!]").yellow().bold(),
                     style(msg).yellow().bold(), width = width
            )
        }
        ReportStatus::ERROR(msg) => {
            writeln!(buffer,
                     "{:>width$} {}",
                     style("[✘]").red().bold(),
                     style(msg).bold(), width = width
            )
        }
        ReportStatus::NA(msg) => {
            writeln!(buffer,
                     "{:>width$} {}",
                     style("[-]").bold(),
                     style(msg).bold(), width = width
            )
        }
    };
}

fn console_report_statuses(report_statuses: &[ReportStatus]) -> String {
    let mut result = String::new();
    if report_statuses.is_empty() {
        return result;
    }
    let mut statuses_iter = report_statuses.iter();
    console_report_status(&mut result, statuses_iter.next().unwrap(), 0);
    for report_status in statuses_iter {
        console_report_status(&mut result, report_status, 2);
    }
    result
}

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
    let connection = display_report_pending(report_pending);

    let connection_data = fatal_if_none(connection.data, "Diagnosis stops here.");

    // Analysis of artifacts
    let report_pending = ArtifactSizeJob::from(&connection_data).diagnose();
    let _ = display_report_pending(report_pending);
}

fn display_report_pending<T: Reportable>(report_pending: ReportPending<T>) -> T {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!(" [*] {}", &report_pending.pending_msg));
    let result = report_pending.job.join().unwrap();
    pb.finish_with_message(console_report_statuses(&result.report()));
    result
}
