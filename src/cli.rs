use std::fmt::Write;
use std::process;
use std::time::Duration;

use console::style;
use indicatif::ProgressBar;
use structopt::StructOpt;

use crate::{Reportable, ReportPending, ReportStatus};

pub fn fatal_if_none<T>(result: Option<T>, msg: &str) -> T {
    match result {
        Some(x) => x,
        None => {
            eprintln!("{}", msg);
            process::exit(1);
        }
    }
}

#[derive(StructOpt)]
pub struct Args {
    #[structopt(name = "url", long)]
    /// Analyze the project from the URL of Gitlab repository
    pub url: Option<String>,
    #[structopt(name = "git_path")]
    /// Analyze the project from a local path of a Git repository
    pub git_path: Option<String>,
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

pub fn display_report_pending<T: Reportable>(report_pending: ReportPending<T>) -> T {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!(" [*] {}", &report_pending.pending_msg));
    let result = report_pending.job.join().unwrap();
    pb.finish_with_message(console_report_statuses(&result.report()));
    eprint!("\r");
    result
}
