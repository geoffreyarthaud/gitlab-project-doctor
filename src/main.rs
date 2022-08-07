use crate::diagnosis::gitlab_connection::GitlabConnection;
use crate::diagnosis::global_storage::GlobalStorage;
use crate::diagnosis::Diagnosis;
use crate::diagnosis::{Report, ReportStatus};
use console::style;
use std::process;

pub mod diagnosis;

fn fatal_if_none<T>(result: Option<T>, msg: &str) -> T {
    match result {
        Some(x) => x,
        None => {
            eprintln!("{msg}");
            process::exit(1);
        }
    }
}

// TODO Make structopt
// #[derive(StructOpt)]
// struct Args {
//     #[structopt(name = "topo-order", long)]
//     /// sort commits in topological order
//     flag_topo_order: bool,
//     #[structopt(name = "date-order", long)]
//     /// sort commits in date order
//     flag_date_order: bool,
//     #[structopt(name = "reverse", long)]
//     /// sort commits in reverse
//     flag_reverse: bool,
//     #[structopt(name = "not")]
//     /// don't show <spec>
//     flag_not: Vec<String>,
//     #[structopt(name = "spec", last = true)]
//     arg_spec: Vec<String>,
// }

fn display_report(report: &Report) {
    match &report.global {
        ReportStatus::OK(msg) => {
            eprintln!("{} {}", style("[✓]").green(), msg);
        }
        ReportStatus::WARNING(msg) => {
            eprintln!("{} {}", style("[!]").yellow().bold(), style(msg).bold());
        }
        ReportStatus::ERROR(msg) => {
            eprintln!("{} {}", style("[✘]").red().bold(), style(msg).bold());
        }
    }
}
fn main() {
    let mut gitlab_connection = GitlabConnection::from_path(".");
    display_report(gitlab_connection.diagnosis());
    let data = fatal_if_none(gitlab_connection.data, "Diagnosis stops here.");

    let mut gitlab_storage = GlobalStorage::new(&data.gitlab, &data.project);
    display_report(gitlab_storage.diagnosis());

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
