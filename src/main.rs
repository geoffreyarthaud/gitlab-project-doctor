use crate::diagnosis::gitlab_connection::{ConnectionJob};
use crate::diagnosis::{Reportable, ReportJob, ReportPending};
use crate::diagnosis::{ReportStatus};
use structopt::StructOpt;

use console::style;
use std::process;
use std::time::Duration;
use indicatif::ProgressBar;

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

fn console_report_status(report: &ReportStatus, indent: usize) -> String {
    let width = indent + 4;
    match &report {
        ReportStatus::OK(msg) => {
            format!("{:>width$} {}", style("[✓]").green(), msg, width = width)
        }
        ReportStatus::WARNING(msg) => {
            format!(
                "{:>width$} {}",
                style("[!]").yellow().bold(),
                style(msg).yellow().bold()
            )
        }
        ReportStatus::ERROR(msg) => {
            format!(
                "{:>width$} {}",
                style("[✘]").red().bold(),
                style(msg).bold()
            )
        }
        ReportStatus::NA(msg) => {
            format!(
                "{:>width$} {}",
                style("[-]").bold(),
                style(msg).bold()
            )
        }
    }
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
    let connection = display_report_pending(report_pending);
    fatal_if_none(connection.data, "Diagnosis stops here.");

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
    while !report_pending.job.is_finished() {
        std::thread::sleep(Duration::from_millis(50));
    }
    let result = report_pending.job.join().unwrap();
    pb.finish_with_message(console_report_status(&result.report(), 0));
    result
}
