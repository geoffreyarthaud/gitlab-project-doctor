use atty::Stream;
use std::fmt::Write;
use std::time::Duration;
use std::{panic, process};

use crate::{fl, ReportPending, ReportStatus, Reportable};
use console::style;
use dialoguer::{Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use structopt::StructOpt;

pub fn fatal_if_none<T>(result: Option<T>, msg: &str) -> T {
    match result {
        Some(x) => x,
        None => {
            eprintln!("{}", msg);
            process::exit(1);
        }
    }
}
lazy_static! {
    static ref HELP_URL: String = fl!("help-url");
    static ref HELP_GIT_PATH: String = fl!("help-git-path");
    static ref HELP_BATCH_MODE: String = fl!("help-batch");
    static ref HELP_DAYS: String = fl!("help-days");
}

#[derive(StructOpt)]
pub struct Args {
    #[structopt(name = "url", long, help = &HELP_URL)]
    pub url: Option<String>,
    #[structopt(name = "git_path", required_unless = "url", help = &HELP_GIT_PATH)]
    pub git_path: Option<String>,
    #[structopt(long = "batch", short = "b", help = &HELP_BATCH_MODE)]
    pub batch_mode: bool,
    #[structopt(long = "days", short = "d", default_value = "30", help = &HELP_DAYS)]
    pub days: usize,
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
    if atty::is(Stream::Stderr) {
        _display_report_pending_with_progress(report_pending)
    } else {
        _display_report_pending_no_progress(report_pending)
    }
}

fn _display_report_pending_no_progress<T: Reportable>(report_pending: ReportPending<T>) -> T {
    if report_pending.progress.is_some() && report_pending.total.is_some() {
        eprintln!("  {}", report_pending.pending_msg);
        let rx = report_pending.progress.unwrap();
        let total = report_pending.total.unwrap();
        let milestone = total / 10;
        while let Ok(received) = rx.recv() {
            if milestone != 0 && received % milestone == 0 {
                eprintln!("  {} %", received * 100 / total);
            }
        }
    } else {
        eprintln!("  {} ...", report_pending.pending_msg);
    }

    let result_join = report_pending.job.join();
    match result_join {
        Ok(result) => {
            eprintln!("{}", console_report_statuses(&result.report(), 2));
            result
        }
        Err(e) => panic::resume_unwind(e),
    }
}

fn _display_report_pending_with_progress<T: Reportable>(report_pending: ReportPending<T>) -> T {
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

pub fn input_clean_artifacts(days: usize) -> Option<usize> {
    if Confirm::new()
        .with_prompt(fl!("ask-delete-pipelines"))
        .interact()
        .unwrap_or(false)
    {
        let input: usize = Input::new()
            .with_prompt(fl!("ask-age-days"))
            .default(days)
            .interact_text()
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
        .with_prompt(fl!("ask-delete-files"))
        .interact()
        .unwrap_or(false)
}
