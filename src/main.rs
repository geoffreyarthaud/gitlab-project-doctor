use gitlab::Gitlab;
use std::env;
use std::process;

fn or_fatal<T, U: std::fmt::Display>(result: Result<T, U>) -> T {
    match result {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

fn main() {
    let token = or_fatal(env::var("GL_TOKEN"));
    let client = or_fatal(Gitlab::new(
        "gitlab-forge.din.developpement-durable.gouv.fr",
        token,
    ));
}
