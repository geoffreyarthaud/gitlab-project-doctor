use crate::diagnosis::gitlab_connection::GitlabConnection;
use crate::diagnosis::global_storage::GlobalStorage;
use crate::diagnosis::Diagnosis;
use crate::diagnosis::{Report, ReportStatus};
use structopt::StructOpt;

use console::style;
use std::process;

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

fn display_report(report: &Report, indent: usize) {
    let width = indent + 4;
    match &report.global {
        ReportStatus::OK(msg) => {
            eprintln!("{:>width$} {}", style("[✓]").green(), msg, width = width);
        }
        ReportStatus::WARNING(msg) => {
            eprintln!(
                "{:>width$} {}",
                style("[!]").yellow().bold(),
                style(msg).yellow().bold()
            );
        }
        ReportStatus::ERROR(msg) => {
            eprintln!(
                "{:>width$} {}",
                style("[✘]").red().bold(),
                style(msg).bold()
            );
        }
        ReportStatus::NA(msg) => {
            eprintln!(
                "{:>width$} {}",
                style("[-]").bold(),
                style(msg).bold()
            );
        }
    }
    for subreport in &report.details {
        display_report(subreport, indent + 4);
    }
}
fn main() {
    let args = Args::from_args();
    let mut gitlab_connection = {
        if args.url.is_some() {
           GitlabConnection::from_url(&args.url.unwrap())
        } else {
            let default_path = String::from(".");
            let path: &str = args.git_path.as_ref().unwrap_or(&default_path);
            GitlabConnection::from_git_path(path)
        }
    };

    display_report(gitlab_connection.diagnosis(), 0);
    let data = fatal_if_none(gitlab_connection.data, "Diagnosis stops here.");

    let mut gitlab_storage = GlobalStorage::new(&data.gitlab, &data.project);
    display_report(gitlab_storage.diagnosis(), 0);

    // println!(
    //     "{} Storage size : {}",
    //     style("[✓]").green(),
    //     human_bytes(gitlab_repo.project.statistics.storage_size as f64)
    // );
    // println!(
    //     "    {} Repository size : {} ({} %)",
    //     style("[✓]").green(),
    //     human_bytes(gitlab_repo.project.statistics.repository_size as f64),
    //     100 * gitlab_repo.project.statistics.repository_size
    //         / gitlab_repo.project.statistics.storage_size
    // );
    // println!(
    //     "    {} Job artifacts size : {} ({} %)",
    //     style("[✓]").green(),
    //     human_bytes(gitlab_repo.project.statistics.job_artifacts_size as f64),
    //     100 * gitlab_repo.project.statistics.job_artifacts_size
    //         / gitlab_repo.project.statistics.storage_size
    // );

    // let mut revs = repo.revwalk().unwrap();
    // revs.push_head().unwrap();
    // for rev in revs {
    //     println!("{}", rev.unwrap());
    // }
}
