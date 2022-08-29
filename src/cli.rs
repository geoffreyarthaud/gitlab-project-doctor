use std::fmt::Write;
use std::time::Duration;
use std::{panic, process};

use console::style;
use dialoguer::{Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};
use structopt::StructOpt;

use crate::{ReportPending, ReportStatus, Reportable};

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
    #[structopt(name = "git_path", required_unless = "url")]
    /// Analyze the project from a local path of a Git repository. Ignored if url option is
    /// specified
    pub git_path: Option<String>,
}

fn console_report_status(buffer: &mut String, report_status: &ReportStatus, indent: usize) {
    let width = indent + 4;
    let _ = match &report_status {
        ReportStatus::OK(msg) => {
            writeln!(
                buffer,
                "{:>width$} {}",
                style("[✓]").green(),
                msg,
                width = width
            )
        }
        ReportStatus::WARNING(msg) => {
            writeln!(
                buffer,
                "{:>width$} {}",
                style("[!]").yellow().bold(),
                style(msg).yellow().bold(),
                width = width
            )
        }
        ReportStatus::ERROR(msg) => {
            writeln!(
                buffer,
                "{:>width$} {}",
                style("[✘]").red().bold(),
                style(msg).bold(),
                width = width
            )
        }
        ReportStatus::NA(msg) => {
            writeln!(
                buffer,
                "{:>width$} {}",
                style("[-]").bold(),
                style(msg).bold(),
                width = width
            )
        }
    };
}

pub fn console_report_statuses(report_statuses: &[ReportStatus], initial_indent: usize) -> String {
    eprint!("\r");
    let mut result = String::new();
    if report_statuses.is_empty() {
        return result;
    }
    let mut statuses_iter = report_statuses.iter();
    console_report_status(&mut result, statuses_iter.next().unwrap(), initial_indent);
    for report_status in statuses_iter {
        console_report_status(&mut result, report_status, 2);
    }
    result
}

pub fn display_report_pending<T: Reportable>(report_pending: ReportPending<T>) -> T {
    let pb;
    let initial_indent: usize;
    if report_pending.progress.is_some() && report_pending.total.is_some() {
        initial_indent = 2;
        let sty = ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} ({eta})")
            .unwrap()
            .progress_chars("#>-");

        pb = ProgressBar::new(report_pending.total.unwrap() as u64);
        pb.set_style(sty);
        pb.set_message(report_pending.pending_msg.to_string());
        let rx = report_pending.progress.unwrap();
        while let Ok(received) = rx.recv() {
            pb.set_position(received as u64);
        }
    } else {
        initial_indent = 0;
        pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message(format!(" [*] {}", &report_pending.pending_msg));
    }

    let result_join = report_pending.job.join();
    match result_join {
        Ok(result) => {
            pb.finish_with_message(console_report_statuses(&result.report(), initial_indent));
            eprint!("\r");
            result
        }
        Err(e) => panic::resume_unwind(e),
    }
}

pub fn input_clean_artifacts() -> Option<i64> {
    if Confirm::new()
        .with_prompt("Delete old pipelines ?")
        .interact()
        .unwrap_or(false)
    {
        let input: i64 = Input::new()
            .with_prompt("From which age in days ?")
            .default("30".into())
            .interact_text()
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);
        if input > 0 {
            Some(input)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn input_clean_files() -> bool {
    Confirm::new()
        .with_prompt("Delete obsolete files ?")
        .interact()
        .unwrap_or(false)
}
