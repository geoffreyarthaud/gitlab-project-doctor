use std::{process};
use std::fmt::Write as _;
use std::time::Duration;

use console::style;
use indicatif::ProgressBar;
use structopt::StructOpt;

use crate::diagnosis::{Reportable, ReportJob, ReportPending};
use crate::diagnosis::ReportStatus;
use crate::diagnosis::gitlab_connection::ConnectionJob;

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

fn console_report_status(report_statuses: &[ReportStatus], indent: usize) -> String {
    let width = indent + 4;
    let mut result = String::new();
    let _ = writeln!(result);
    for report in report_statuses.iter() {
        let _ = match &report {
            ReportStatus::OK(msg) => {
                writeln!(result, "{:>width$} {}", style("[✓]").green(), msg, width = width)
            }
            ReportStatus::WARNING(msg) => {
                writeln!(result,
                         "{:>width$} {}",
                         style("[!]").yellow().bold(),
                         style(msg).yellow().bold(), width = width
                )
            }
            ReportStatus::ERROR(msg) => {
                writeln!(result,
                         "{:>width$} {}",
                         style("[✘]").red().bold(),
                         style(msg).bold(), width = width
                )
            }
            ReportStatus::NA(msg) => {
                writeln!(result,
                         "{:>width$} {}",
                         style("[-]").bold(),
                         style(msg).bold(), width = width
                )
            }
        };
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
    let report_pending = connection_job.diagnose();
    let _ = display_report_pending(report_pending);
    // let connection_data = fatal_if_none(connection.data, "Diagnosis stops here.");

    //display_report_pending(RepoStorageJob::from(&connection_data.project).diagnose());


    //
    // let mut gitlab_storage = GlobalStorage::new(&data.gitlab, &data.project);
    // display_report(gitlab_storage.diagnosis(), 0);


    // let mut revs = repo.revwalk().unwrap();
    // revs.push_head().unwrap();
    // for rev in revs {
    //     println!("{}", rev.unwrap());
    // }
}

fn display_report_pending<T: Reportable>(report_pending: ReportPending<T>) -> T {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(50));
    pb.set_message(format!("[*] {}", &report_pending.pending_msg));
    let result = report_pending.job.join().unwrap();
    pb.finish_with_message(console_report_status(&result.report(), 0));
    result
}
